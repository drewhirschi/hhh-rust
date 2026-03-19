use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    InternalError(String),
    NotFound,
    Forbidden,
    BadRequest(String),
    Redirect(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Forbidden => StatusCode::FORBIDDEN.into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Redirect(url) => {
                (StatusCode::FOUND, [(header::LOCATION, url)]).into_response()
            }
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InternalError(msg) => write!(f, "Internal error: {msg}"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            AppError::Redirect(url) => write!(f, "Redirect to {url}"),
        }
    }
}
