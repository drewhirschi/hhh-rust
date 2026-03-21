pub mod account;
pub mod admin;
pub mod bookings;
pub mod classes;
pub mod dashboard;
pub mod employee;
pub mod public;

use axum::{extract::State, http::StatusCode, routing::get, Router};
use crate::AppState;

async fn ready(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .merge(public::router())
        .merge(dashboard::router())
        .merge(classes::router())
        .merge(bookings::router())
        .merge(account::router())
        .merge(employee::router())
        .merge(admin::router())
        .with_state(state)
}
