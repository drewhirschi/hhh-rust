use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use tower_cookies::{Cookie, Cookies};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::booking::Booking;
use crate::models::class::{ClassSchedule, ScheduleView};
use crate::models::user::User;
use crate::AppState;

fn get_flash(cookies: &Cookies) -> Option<String> {
    let val = cookies.get("flash").map(|c| c.value().to_string());
    if val.is_some() {
        cookies.remove(Cookie::from("flash"));
    }
    val
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/classes", get(schedule_list))
        .route("/classes/{id}", get(class_detail))
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "classes/schedule.html")]
struct ScheduleListTemplate {
    schedules: Vec<ScheduleView>,
    booked_ids: Vec<i64>,
    user: Option<User>,
    flash: Option<String>,
}

async fn schedule_list(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<ScheduleListTemplate, AppError> {
    let schedules = ClassSchedule::list_upcoming(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let mut booked_ids = Vec::new();
    for schedule in &schedules {
        let is_booked = Booking::is_user_booked(&state.db, schedule.id, user.id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        if is_booked {
            booked_ids.push(schedule.id);
        }
    }

    let flash = get_flash(&cookies);

    Ok(ScheduleListTemplate {
        schedules,
        booked_ids,
        user: Some(user),
        flash,
    })
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "classes/detail.html")]
struct ClassDetailTemplate {
    schedule: ScheduleView,
    is_booked: bool,
    booking_id: i64,
    user: Option<User>,
    flash: Option<String>,
}

async fn class_detail(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    cookies: Cookies,
) -> Result<ClassDetailTemplate, AppError> {
    let schedule: ScheduleView = sqlx::query_as(
        "SELECT
            cs.id,
            cd.name AS class_name,
            cd.description,
            u.display_name AS instructor_name,
            cs.starts_at,
            cs.ends_at,
            cd.capacity,
            cs.is_cancelled,
            COALESCE((SELECT COUNT(*) FROM bookings b WHERE b.class_schedule_id = cs.id AND b.status = 'confirmed'), 0) AS booked_count
         FROM class_schedules cs
         JOIN class_definitions cd ON cd.id = cs.class_definition_id
         LEFT JOIN users u ON u.id = cs.instructor_id
         WHERE cs.id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let is_booked = Booking::is_user_booked(&state.db, id, user.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let booking_id = if is_booked {
        Booking::find_by_user_and_schedule(&state.db, user.id, id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .map(|b| b.id)
            .unwrap_or(0)
    } else {
        0
    };

    let flash = get_flash(&cookies);

    Ok(ClassDetailTemplate {
        schedule,
        is_booked,
        booking_id,
        user: Some(user),
        flash,
    })
}
