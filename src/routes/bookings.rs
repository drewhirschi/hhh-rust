use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Form, Router};
use tower_cookies::{Cookie, Cookies};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::booking::{Booking, BookingView};
use crate::models::user::User;
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
        .route("/bookings", get(my_bookings).post(create_booking))
        .route("/bookings/{id}/cancel", axum::routing::post(cancel_booking))
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "bookings/my_bookings.html")]
struct MyBookingsTemplate {
    bookings: Vec<BookingView>,
    user: Option<User>,
    flash: Option<String>,
}

async fn my_bookings(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<MyBookingsTemplate, AppError> {
    let bookings = Booking::list_for_user(&state.db, user.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let flash = get_flash(&cookies);

    Ok(MyBookingsTemplate {
        bookings,
        user: Some(user),
        flash,
    })
}

#[derive(serde::Deserialize)]
struct CreateBookingForm {
    class_schedule_id: i64,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "bookings/_booking_row.html")]
struct BookingRowTemplate {
    booking: BookingView,
}

async fn create_booking(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Form(form): Form<CreateBookingForm>,
) -> Result<Response, AppError> {
    Booking::create_booking(&state.db, form.class_schedule_id, user.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let is_htmx = headers.get("HX-Request").is_some();

    if is_htmx {
        return Ok((StatusCode::OK, [("HX-Redirect", "/classes")]).into_response());
    }

    set_flash(&cookies, "Booking confirmed!");
    Ok(Redirect::to("/classes").into_response())
}

async fn cancel_booking(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    cookies: Cookies,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    Booking::cancel_booking(&state.db, id, user.id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let is_htmx = headers.get("HX-Request").is_some();

    if is_htmx {
        // Re-fetch the booking as a BookingView to render the updated row
        let bookings = Booking::list_for_user(&state.db, user.id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if let Some(booking) = bookings.into_iter().find(|b| b.id == id) {
            return Ok(BookingRowTemplate { booking }.into_response());
        }

        return Ok((StatusCode::OK, [("HX-Redirect", "/bookings")]).into_response());
    }

    set_flash(&cookies, "Booking cancelled.");
    Ok(Redirect::to("/bookings").into_response())
}
