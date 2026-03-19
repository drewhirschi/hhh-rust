use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tower_cookies::Cookies;

use crate::error::AppError;
use crate::models::user::User;
use crate::AppState;

use super::session::get_session_user;

pub struct AuthUser(pub User);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, &())
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to extract cookies: {e:?}")))?;

        let session_id = cookies
            .get("session_id")
            .map(|c| c.value().to_string())
            .ok_or_else(|| AppError::Redirect("/login".to_string()))?;

        let user = get_session_user(&state.db, &session_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Session lookup failed: {e}")))?
            .ok_or_else(|| AppError::Redirect("/login".to_string()))?;

        Ok(AuthUser(user))
    }
}

pub struct RequireEmployee(pub User);

impl FromRequestParts<AppState> for RequireEmployee {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_employee() {
            return Err(AppError::Forbidden);
        }

        Ok(RequireEmployee(user))
    }
}

pub struct RequireAdmin(pub User);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_admin() {
            return Err(AppError::Forbidden);
        }

        Ok(RequireAdmin(user))
    }
}
