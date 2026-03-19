use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireAdmin,
    auth::password::hash_password,
    error::AppError,
    models::user::{self, User},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users).post(create_user))
        .route("/admin/users/new", get(new_user_form))
        .route("/admin/users/{id}/edit", get(edit_user_form))
        .route("/admin/users/{id}", post(update_user))
        .route("/admin/users/{id}/toggle", post(toggle_user))
}

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

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "admin/users.html")]
struct UsersTemplate {
    users: Vec<User>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "admin/user_form.html")]
struct UserFormTemplate {
    edit_user: Option<User>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(serde::Deserialize)]
struct CreateUserForm {
    email: String,
    display_name: String,
    role: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct UpdateUserForm {
    email: String,
    display_name: String,
    role: String,
}

async fn list_users(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let users = user::list_all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(UsersTemplate {
        users,
        user: Some(current_user),
        flash,
    })
}

async fn new_user_form(
    RequireAdmin(current_user): RequireAdmin,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);
    Ok(UserFormTemplate {
        edit_user: None,
        user: Some(current_user),
        flash,
    })
}

async fn create_user(
    State(state): State<AppState>,
    RequireAdmin(_current_user): RequireAdmin,
    cookies: Cookies,
    Form(form): Form<CreateUserForm>,
) -> Result<impl IntoResponse, AppError> {
    let password_hash = hash_password(&form.password)?;

    user::create_user(
        &state.db,
        &form.email,
        &password_hash,
        &form.display_name,
        &form.role,
    )
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "User created successfully");
    Ok(Redirect::to("/admin/users"))
}

async fn edit_user_form(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let edit_user = user::find_by_id(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

    Ok(UserFormTemplate {
        edit_user: Some(edit_user),
        user: Some(current_user),
        flash,
    })
}

async fn update_user(
    State(state): State<AppState>,
    RequireAdmin(_current_user): RequireAdmin,
    cookies: Cookies,
    Path(id): Path<i64>,
    Form(form): Form<UpdateUserForm>,
) -> Result<impl IntoResponse, AppError> {
    user::update_user(&state.db, id, &form.email, &form.display_name, &form.role)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "User updated successfully");
    Ok(Redirect::to("/admin/users"))
}

async fn toggle_user(
    State(state): State<AppState>,
    RequireAdmin(_current_user): RequireAdmin,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    user::toggle_active(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "User status updated");
    Ok(Redirect::to("/admin/users"))
}
