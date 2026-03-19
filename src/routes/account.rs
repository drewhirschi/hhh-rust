use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Form, Router};
use tower_cookies::{Cookie, Cookies};

use crate::auth::password::{hash_password, verify_password};
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::user::{self, User};
use crate::AppState;

fn get_flash(cookies: &Cookies) -> Option<String> {
    let val = cookies.get("flash").map(|c| c.value().to_string());
    if val.is_some() {
        cookies.remove(Cookie::from("flash"));
    }
    val
}

fn set_flash(cookies: &Cookies, msg: &str) {
    cookies.add(Cookie::new("flash", msg.to_string()));
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/account", get(profile_page).post(profile_update))
        .route(
            "/account/password",
            get(password_page).post(password_update),
        )
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "account/profile.html")]
struct ProfileTemplate {
    profile: User,
    user: Option<User>,
    flash: Option<String>,
}

async fn profile_page(
    AuthUser(user): AuthUser,
    cookies: Cookies,
) -> Result<ProfileTemplate, AppError> {
    let flash = get_flash(&cookies);

    Ok(ProfileTemplate {
        profile: user.clone(),
        user: Some(user),
        flash,
    })
}

#[derive(serde::Deserialize)]
struct ProfileUpdateForm {
    display_name: String,
    email: String,
}

async fn profile_update(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<ProfileUpdateForm>,
) -> Result<Response, AppError> {
    user::update_user(&state.db, user.id, &form.email, &form.display_name, &user.role)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Profile updated successfully.");
    Ok(Redirect::to("/account").into_response())
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "account/change_password.html")]
struct ChangePasswordTemplate {
    user: Option<User>,
    flash: Option<String>,
}

async fn password_page(
    AuthUser(user): AuthUser,
    cookies: Cookies,
) -> Result<ChangePasswordTemplate, AppError> {
    let flash = get_flash(&cookies);

    Ok(ChangePasswordTemplate {
        user: Some(user),
        flash,
    })
}

#[derive(serde::Deserialize)]
struct PasswordUpdateForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

async fn password_update(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<PasswordUpdateForm>,
) -> Result<Response, AppError> {
    let is_valid = verify_password(&form.current_password, &user.password_hash)?;
    if !is_valid {
        set_flash(&cookies, "Current password is incorrect.");
        return Ok(Redirect::to("/account/password").into_response());
    }

    if form.new_password != form.confirm_password {
        set_flash(&cookies, "New passwords do not match.");
        return Ok(Redirect::to("/account/password").into_response());
    }

    if form.new_password.is_empty() {
        set_flash(&cookies, "New password cannot be empty.");
        return Ok(Redirect::to("/account/password").into_response());
    }

    let new_hash = hash_password(&form.new_password)?;
    user::update_password(&state.db, user.id, &new_hash)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Password changed successfully.");
    Ok(Redirect::to("/account").into_response())
}
