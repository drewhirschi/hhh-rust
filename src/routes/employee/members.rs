use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireEmployee,
    error::AppError,
    models::user::{self, User},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/employee/members", get(list_members))
        .route("/employee/members/{id}/toggle", post(toggle_member))
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
#[template(path = "employee/members.html")]
struct MembersTemplate {
    members: Vec<User>,
    user: Option<User>,
    flash: Option<String>,
}

async fn list_members(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let members: Vec<User> = sqlx::query_as(
        "SELECT * FROM users WHERE role = 'member' ORDER BY display_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(MembersTemplate {
        members,
        user: Some(user),
        flash,
    })
}

async fn toggle_member(
    State(state): State<AppState>,
    RequireEmployee(_user): RequireEmployee,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    user::toggle_active(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Member status updated");
    Ok(Redirect::to("/employee/members"))
}
