use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_employee(&self) -> bool {
        self.role == "employee" || self.role == "admin"
    }

    pub fn is_member(&self) -> bool {
        true
    }
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(
    pool: &SqlitePool,
    email: &str,
    password_hash: &str,
    display_name: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (email, password_hash, display_name, role, is_active, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 1, datetime('now'), datetime('now'))",
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(role)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    // Safe to unwrap: we just inserted this row
    find_by_id(pool, id).await.map(|opt| opt.unwrap())
}

pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    email: &str,
    display_name: &str,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET email = ?, display_name = ?, role = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(email)
    .bind(display_name)
    .bind(role)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC")
        .fetch_all(pool)
        .await
}

pub async fn toggle_active(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET is_active = NOT is_active, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_password(
    pool: &SqlitePool,
    id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_all(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}
