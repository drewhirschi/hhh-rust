pub mod account;
pub mod admin;
pub mod bookings;
pub mod classes;
pub mod dashboard;
pub mod employee;
pub mod public;

use axum::Router;
use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(public::router())
        .merge(dashboard::router())
        .merge(classes::router())
        .merge(bookings::router())
        .merge(account::router())
        .merge(employee::router())
        .merge(admin::router())
        .with_state(state)
}
