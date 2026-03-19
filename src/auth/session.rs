use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn create_session(pool: &SqlitePool, user_id: i64) -> Result<String, sqlx::Error> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339();

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(user_id)
        .bind(&expires_at)
        .execute(pool)
        .await?;

    Ok(session_id)
}

pub async fn get_session_user(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u \
         INNER JOIN sessions s ON s.user_id = u.id \
         WHERE s.id = ? AND s.expires_at > ? AND u.is_active = true",
    )
    .bind(session_id)
    .bind(&now)
    .fetch_optional(pool)
    .await
}

pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(&now)
        .execute(pool)
        .await?;
    Ok(())
}
