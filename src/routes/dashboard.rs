use axum::{
    extract::State,
    response::IntoResponse,
    Router,
    routing::get,
};
use tower_cookies::Cookies;

use crate::{auth::AuthUser, error::AppError, models::user::User, AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/dashboard", get(dashboard))
}

fn get_flash(cookies: &Cookies) -> Option<String> {
    let val = cookies.get("flash").map(|c| c.value().to_string());
    if val.is_some() {
        cookies.remove(tower_cookies::Cookie::from("flash"));
    }
    val
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "dashboard/member.html")]
struct MemberDashboardTemplate {
    user: Option<User>,
    flash: Option<String>,
    user_name: String,
    upcoming_count: i64,
    total_bookings: i64,
    available_classes: i64,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "dashboard/employee.html")]
struct EmployeeDashboardTemplate {
    user: Option<User>,
    flash: Option<String>,
    total_members: i64,
    active_classes: i64,
    todays_classes: i64,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "dashboard/admin.html")]
struct AdminDashboardTemplate {
    user: Option<User>,
    flash: Option<String>,
    total_users: i64,
    active_sessions: i64,
    total_bookings: i64,
    active_classes: i64,
}

async fn dashboard(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    if user.is_admin() {
        let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let active_sessions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE expires_at > datetime('now')",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let total_bookings: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bookings")
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let active_classes: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM class_definitions WHERE is_active = 1",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        Ok(AdminDashboardTemplate {
            user: Some(user),
            flash,
            total_users: total_users.0,
            active_sessions: active_sessions.0,
            total_bookings: total_bookings.0,
            active_classes: active_classes.0,
        }
        .into_response())
    } else if user.is_employee() {
        let total_members: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE role = 'member' AND is_active = 1",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let active_classes: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM class_definitions WHERE is_active = 1",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let todays_classes: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM class_schedules WHERE date(starts_at) = date('now') AND is_cancelled = 0",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        Ok(EmployeeDashboardTemplate {
            user: Some(user),
            flash,
            total_members: total_members.0,
            active_classes: active_classes.0,
            todays_classes: todays_classes.0,
        }
        .into_response())
    } else {
        let upcoming_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM bookings b \
             JOIN class_schedules cs ON cs.id = b.class_schedule_id \
             WHERE b.user_id = ? AND b.status = 'confirmed' AND cs.starts_at > datetime('now')",
        )
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let total_bookings: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM bookings WHERE user_id = ?",
        )
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let available_classes: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM class_schedules WHERE starts_at > datetime('now') AND is_cancelled = 0",
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("Database error: {e}")))?;

        let user_name = user.display_name.clone();

        Ok(MemberDashboardTemplate {
            user: Some(user),
            flash,
            user_name,
            upcoming_count: upcoming_count.0,
            total_bookings: total_bookings.0,
            available_classes: available_classes.0,
        }
        .into_response())
    }
}
