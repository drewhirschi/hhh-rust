pub mod classes;
pub mod members;
pub mod invites;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(classes::router())
        .merge(members::router())
        .merge(invites::router())
}
