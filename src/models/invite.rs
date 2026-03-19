use rand::Rng;
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Invite {
    pub id: i64,
    pub code: String,
    pub created_by: i64,
    pub used_by: Option<i64>,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InviteView {
    pub id: i64,
    pub code: String,
    pub created_by: i64,
    pub creator_name: String,
    pub used_by: Option<i64>,
    pub expires_at: String,
    pub created_at: String,
}

fn generate_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

impl Invite {
    pub async fn create(pool: &SqlitePool, created_by: i64) -> Result<Invite, sqlx::Error> {
        let code = generate_code();
        sqlx::query_as::<_, Invite>(
            "INSERT INTO invites (code, created_by, expires_at, created_at)
             VALUES (?, ?, datetime('now', '+7 days'), datetime('now'))
             RETURNING id, code, created_by, used_by, expires_at, created_at",
        )
        .bind(&code)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_code(pool: &SqlitePool, code: &str) -> Result<Option<Invite>, sqlx::Error> {
        sqlx::query_as::<_, Invite>(
            "SELECT id, code, created_by, used_by, expires_at, created_at
             FROM invites
             WHERE code = ?",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn mark_used(pool: &SqlitePool, id: i64, used_by: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE invites SET used_by = ? WHERE id = ?")
            .bind(used_by)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<InviteView>, sqlx::Error> {
        sqlx::query_as::<_, InviteView>(
            "SELECT
                i.id,
                i.code,
                i.created_by,
                u.display_name AS creator_name,
                i.used_by,
                i.expires_at,
                i.created_at
             FROM invites i
             JOIN users u ON u.id = i.created_by
             ORDER BY i.created_at DESC",
        )
        .fetch_all(pool)
        .await
    }
}
