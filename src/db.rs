use argon2::{
    password_hash::SaltString,
    Argon2, PasswordHasher,
};
use rand::rngs::OsRng;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Executor, SqlitePool};
use tracing::info;
use std::str::FromStr;

use crate::config::Config;

pub async fn init_pool(config: &Config) -> SqlitePool {
    let opts = SqliteConnectOptions::from_str(&config.database_url)
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .expect("Failed to connect to database");

    run_migrations(&pool).await;
    seed_admin(&pool).await;

    pool
}

async fn run_migrations(pool: &SqlitePool) {
    let migration_sql = include_str!("../migrations/001_initial.sql");

    // SQLite can execute multiple statements via raw_sql
    pool.execute(sqlx::raw_sql(migration_sql))
        .await
        .expect("Failed to run migrations");

    info!("Migrations applied successfully");
}

async fn seed_admin(pool: &SqlitePool) {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@gym.local')")
            .fetch_one(pool)
            .await
            .expect("Failed to check for admin user");

    if exists {
        info!("Admin user already exists, skipping seed");
        return;
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(b"changeme", &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        "INSERT INTO users (email, password_hash, display_name, role) VALUES (?, ?, ?, ?)",
    )
    .bind("admin@gym.local")
    .bind(&password_hash)
    .bind("Admin")
    .bind("admin")
    .execute(pool)
    .await
    .expect("Failed to seed admin user");

    info!("Seeded admin user (admin@gym.local)");
}
