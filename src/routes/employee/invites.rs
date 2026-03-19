use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireEmployee,
    error::AppError,
    models::{invite::{Invite, InviteView}, user::User},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/employee/invites", get(list_invites).post(create_invite))
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
#[template(path = "employee/invites.html")]
struct InvitesTemplate {
    invites: Vec<InviteView>,
    user: Option<User>,
    flash: Option<String>,
}

async fn list_invites(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let invites = Invite::list_all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(InvitesTemplate {
        invites,
        user: Some(user),
        flash,
    })
}

async fn create_invite(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let invite = Invite::create(&state.db, user.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, &format!("Invite created! Code: {}", invite.code));
    Ok(Redirect::to("/employee/invites"))
}
