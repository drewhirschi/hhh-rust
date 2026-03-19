pub mod users;
pub mod settings;
pub mod stats;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(users::router())
        .merge(settings::router())
        .merge(stats::router())
}
