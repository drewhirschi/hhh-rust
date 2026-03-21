use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    info!("Starting server on {}:{}", config.host, config.port);

    let pool = db::init_pool(&config).await;
    let state = AppState { db: pool };

    let app = routes::router(state).layer(CookieManagerLayer::new());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    info!("Listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    info!("Shutdown signal received, draining connections...");
}
