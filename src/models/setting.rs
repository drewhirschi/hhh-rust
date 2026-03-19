use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

impl Setting {
    pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<Setting> = sqlx::query_as::<_, Setting>(
            "SELECT key, value FROM settings WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|s| s.value))
    }

    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Setting>, sqlx::Error> {
        sqlx::query_as::<_, Setting>(
            "SELECT key, value FROM settings ORDER BY key",
        )
        .fetch_all(pool)
        .await
    }
}
