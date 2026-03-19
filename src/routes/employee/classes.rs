use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireEmployee,
    error::AppError,
    models::{
        booking::{Booking, BookingWithUser},
        class::{ClassDefinition, ClassSchedule, ScheduleView},
        user::User,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/employee/classes", get(list_classes).post(create_class))
        .route("/employee/classes/new", get(new_class_form))
        .route(
            "/employee/classes/{id}/edit",
            get(edit_class_form),
        )
        .route("/employee/classes/{id}", post(update_class))
        .route("/employee/classes/{id}/delete", post(delete_class))
        .route("/employee/classes/schedule/new", get(new_schedule_form))
        .route("/employee/classes/schedule", post(create_schedule))
        .route("/employee/rosters/{id}", get(view_roster))
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

// --- Templates ---

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "employee/classes.html")]
struct ClassesTemplate {
    classes: Vec<ClassDefinition>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "employee/class_form.html")]
struct ClassFormTemplate {
    class: Option<ClassDefinition>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "employee/schedule_form.html")]
struct ScheduleFormTemplate {
    classes: Vec<ClassDefinition>,
    instructors: Vec<User>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "employee/rosters.html")]
struct RostersTemplate {
    schedule: ScheduleView,
    attendees: Vec<BookingWithUser>,
    user: Option<User>,
    flash: Option<String>,
}

// --- Form structs ---

#[derive(serde::Deserialize)]
struct ClassForm {
    name: String,
    description: String,
    capacity: i64,
    duration_minutes: i64,
}

#[derive(serde::Deserialize)]
struct ScheduleForm {
    class_definition_id: i64,
    instructor_id: Option<i64>,
    starts_at: String,
}

// --- Handlers ---

async fn list_classes(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);
    let classes = ClassDefinition::list_all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ClassesTemplate {
        classes,
        user: Some(user),
        flash,
    })
}

async fn new_class_form(
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);
    Ok(ClassFormTemplate {
        class: None,
        user: Some(user),
        flash,
    })
}

async fn create_class(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
    Form(form): Form<ClassForm>,
) -> Result<impl IntoResponse, AppError> {
    ClassDefinition::create(
        &state.db,
        &form.name,
        &form.description,
        form.capacity,
        form.duration_minutes,
        user.id,
    )
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Class created successfully");
    Ok(Redirect::to("/employee/classes"))
}

async fn edit_class_form(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);
    let class = ClassDefinition::find_by_id(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or(AppError::NotFound)?;

    Ok(ClassFormTemplate {
        class: Some(class),
        user: Some(user),
        flash,
    })
}

async fn update_class(
    State(state): State<AppState>,
    RequireEmployee(_user): RequireEmployee,
    cookies: Cookies,
    Path(id): Path<i64>,
    Form(form): Form<ClassForm>,
) -> Result<impl IntoResponse, AppError> {
    ClassDefinition::update(
        &state.db,
        id,
        &form.name,
        &form.description,
        form.capacity,
        form.duration_minutes,
    )
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Class updated successfully");
    Ok(Redirect::to("/employee/classes"))
}

async fn delete_class(
    State(state): State<AppState>,
    RequireEmployee(_user): RequireEmployee,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ClassDefinition::delete(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Class deactivated successfully");
    Ok(Redirect::to("/employee/classes"))
}

async fn new_schedule_form(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let classes = ClassDefinition::list_active(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let instructors: Vec<User> = sqlx::query_as(
        "SELECT * FROM users WHERE role IN ('employee', 'admin') AND is_active = 1 ORDER BY display_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(ScheduleFormTemplate {
        classes,
        instructors,
        user: Some(user),
        flash,
    })
}

async fn create_schedule(
    State(state): State<AppState>,
    RequireEmployee(_user): RequireEmployee,
    cookies: Cookies,
    Form(form): Form<ScheduleForm>,
) -> Result<impl IntoResponse, AppError> {
    let class_def = ClassDefinition::find_by_id(&state.db, form.class_definition_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or(AppError::BadRequest("Class not found".to_string()))?;

    let starts_at = chrono::NaiveDateTime::parse_from_str(&form.starts_at, "%Y-%m-%dT%H:%M")
        .map_err(|e| AppError::BadRequest(format!("Invalid date format: {e}")))?;

    let ends_at = starts_at + chrono::Duration::minutes(class_def.duration_minutes);
    let ends_at_str = ends_at.format("%Y-%m-%dT%H:%M").to_string();
    let starts_at_str = starts_at.format("%Y-%m-%dT%H:%M").to_string();

    ClassSchedule::create(
        &state.db,
        form.class_definition_id,
        form.instructor_id,
        &starts_at_str,
        &ends_at_str,
    )
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?;

    set_flash(&cookies, "Schedule created successfully");
    Ok(Redirect::to("/employee/classes"))
}

async fn view_roster(
    State(state): State<AppState>,
    RequireEmployee(user): RequireEmployee,
    cookies: Cookies,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let schedule = sqlx::query_as::<_, ScheduleView>(
        "SELECT cs.id, cd.name AS class_name, cd.description, u.display_name AS instructor_name,
                cs.starts_at, cs.ends_at, cd.capacity, cs.is_cancelled,
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

    let attendees = Booking::list_for_schedule(&state.db, id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(RostersTemplate {
        schedule,
        attendees,
        user: Some(user),
        flash,
    })
}
