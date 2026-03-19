use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form, Router,
    routing::{get, post},
};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use crate::{auth, error::AppError, models, AppState};
use crate::models::user::User;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login_submit))
        .route("/logout", post(logout))
        .route("/register/{code}", get(register_page).post(register_submit))
}

fn get_flash(cookies: &Cookies) -> Option<String> {
    let val = cookies.get("flash").map(|c| c.value().to_string());
    if val.is_some() {
        cookies.remove(Cookie::from("flash"));
    }
    val
}

// GET / -> redirect to /dashboard (dashboard handler will redirect to /login if not authenticated)
async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

// Templates

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "auth/login.html")]
struct LoginTemplate {
    user: Option<User>,
    flash: Option<String>,
    error: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "auth/register.html")]
struct RegisterTemplate {
    user: Option<User>,
    flash: Option<String>,
    error: Option<String>,
    code: String,
}

// Forms

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    display_name: String,
    email: String,
    password: String,
    password_confirm: String,
}

// GET /login
async fn login_page(cookies: Cookies) -> LoginTemplate {
    let flash = get_flash(&cookies);
    LoginTemplate {
        user: None,
        flash,
        error: None,
    }
}

// POST /login
async fn login_submit(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = models::user::find_by_email(&state.db, &form.email)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

    let user = match user {
        Some(u) => u,
        None => {
            return Ok(LoginTemplate {
                user: None,
                flash: None,
                error: Some("Invalid email or password.".to_string()),
            }
            .into_response());
        }
    };

    if !user.is_active {
        return Ok(LoginTemplate {
            user: None,
            flash: None,
            error: Some("Your account has been deactivated.".to_string()),
        }
        .into_response());
    }

    let valid = auth::password::verify_password(&form.password, &user.password_hash)?;
    if !valid {
        return Ok(LoginTemplate {
            user: None,
            flash: None,
            error: Some("Invalid email or password.".to_string()),
        }
        .into_response());
    }

    let session_id = auth::session::create_session(&state.db, user.id)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create session: {e}")))?;

    let mut cookie = Cookie::new("session_id", session_id);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(Redirect::to("/dashboard").into_response())
}

// POST /logout
async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Redirect, AppError> {
    if let Some(session_cookie) = cookies.get("session_id") {
        let session_id = session_cookie.value().to_string();
        auth::session::delete_session(&state.db, &session_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to delete session: {e}")))?;
    }

    let mut cookie = Cookie::from("session_id");
    cookie.set_path("/");
    cookies.remove(cookie);

    Ok(Redirect::to("/login"))
}

// GET /register/{code}
async fn register_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let invite = models::invite::Invite::find_by_code(&state.db, &code)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

    let invite = match invite {
        Some(i) => i,
        None => return Err(AppError::BadRequest("Invalid invite code.".to_string())),
    };

    if invite.used_by.is_some() {
        return Err(AppError::BadRequest("This invite code has already been used.".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    if invite.expires_at < now {
        return Err(AppError::BadRequest("This invite code has expired.".to_string()));
    }

    Ok(RegisterTemplate {
        user: None,
        flash,
        error: None,
        code,
    })
}

// POST /register/{code}
async fn register_submit(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(code): Path<String>,
    Form(form): Form<RegisterForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate invite
    let invite = models::invite::Invite::find_by_code(&state.db, &code)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

    let invite = match invite {
        Some(i) => i,
        None => return Err(AppError::BadRequest("Invalid invite code.".to_string())),
    };

    if invite.used_by.is_some() {
        return Err(AppError::BadRequest("This invite code has already been used.".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    if invite.expires_at < now {
        return Err(AppError::BadRequest("This invite code has expired.".to_string()));
    }

    // Validate form
    if form.password != form.password_confirm {
        return Ok(RegisterTemplate {
            user: None,
            flash: None,
            error: Some("Passwords do not match.".to_string()),
            code,
        }
        .into_response());
    }

    if form.password.len() < 8 {
        return Ok(RegisterTemplate {
            user: None,
            flash: None,
            error: Some("Password must be at least 8 characters.".to_string()),
            code,
        }
        .into_response());
    }

    // Check if email already exists
    let existing = models::user::find_by_email(&state.db, &form.email)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

    if existing.is_some() {
        return Ok(RegisterTemplate {
            user: None,
            flash: None,
            error: Some("An account with that email already exists.".to_string()),
            code,
        }
        .into_response());
    }

    // Create user
    let password_hash = auth::password::hash_password(&form.password)?;
    let user = models::user::create_user(
        &state.db,
        &form.email,
        &password_hash,
        &form.display_name,
        "member",
    )
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to create user: {e}")))?;

    // Mark invite as used
    models::invite::Invite::mark_used(&state.db, invite.id, user.id)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to mark invite used: {e}")))?;

    // Create session
    let session_id = auth::session::create_session(&state.db, user.id)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to create session: {e}")))?;

    let mut cookie = Cookie::new("session_id", session_id);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(Redirect::to("/dashboard").into_response())
}
