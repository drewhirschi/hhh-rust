use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireAdmin,
    error::AppError,
    models::user::User,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/admin/stats", get(show_stats))
}

fn get_flash(cookies: &Cookies) -> Option<String> {
    let val = cookies.get("flash").map(|c| c.value().to_string());
    if val.is_some() {
        cookies.remove(Cookie::from("flash"));
    }
    val
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "admin/stats.html")]
struct StatsTemplate {
    admin_count: i64,
    employee_count: i64,
    member_count: i64,
    total_users: i64,
    active_classes: i64,
    total_bookings: i64,
    active_bookings: i64,
    active_sessions: i64,
    upcoming_schedules: i64,
    user: Option<User>,
    flash: Option<String>,
}

async fn show_stats(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);
    let db = &state.db;
    let map_err = |e: sqlx::Error| AppError::InternalError(e.to_string());

    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin'")
        .fetch_one(db)
        .await
        .map_err(map_err)?;

    let employee_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'employee'")
            .fetch_one(db)
            .await
            .map_err(map_err)?;

    let member_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'member'")
        .fetch_one(db)
        .await
        .map_err(map_err)?;

    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await
        .map_err(map_err)?;

    let active_classes: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM class_definitions WHERE is_active = 1")
            .fetch_one(db)
            .await
            .map_err(map_err)?;

    let total_bookings: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bookings")
        .fetch_one(db)
        .await
        .map_err(map_err)?;

    let active_bookings: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM bookings WHERE status = 'confirmed'")
            .fetch_one(db)
            .await
            .map_err(map_err)?;

    let active_sessions: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE expires_at > datetime('now')")
            .fetch_one(db)
            .await
            .map_err(map_err)?;

    let upcoming_schedules: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM class_schedules WHERE starts_at > datetime('now') AND is_cancelled = 0",
    )
    .fetch_one(db)
    .await
    .map_err(map_err)?;

    Ok(StatsTemplate {
        admin_count: admin_count.0,
        employee_count: employee_count.0,
        member_count: member_count.0,
        total_users: total_users.0,
        active_classes: active_classes.0,
        total_bookings: total_bookings.0,
        active_bookings: active_bookings.0,
        active_sessions: active_sessions.0,
        upcoming_schedules: upcoming_schedules.0,
        user: Some(current_user),
        flash,
    })
}
