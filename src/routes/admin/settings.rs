use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth::RequireAdmin,
    error::AppError,
    models::{setting::Setting, user::User},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/settings", get(show_settings).post(save_settings))
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

struct SettingDisplay {
    key: String,
    value: String,
    description: Option<String>,
}

#[derive(askama::Template, askama_web::WebTemplate)]
#[template(path = "admin/settings.html")]
struct SettingsTemplate {
    settings: Vec<SettingDisplay>,
    user: Option<User>,
    flash: Option<String>,
}

#[derive(serde::Deserialize)]
struct SettingsForm {
    #[serde(default, rename = "keys[]")]
    keys: Vec<String>,
    #[serde(default, rename = "values[]")]
    values: Vec<String>,
}

fn description_for_key(key: &str) -> Option<String> {
    match key {
        "gym_name" => Some("The name of your gym".to_string()),
        "gym_description" => Some("A short description of your gym".to_string()),
        _ => None,
    }
}

async fn show_settings(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let flash = get_flash(&cookies);

    let mut raw_settings = Setting::list_all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if raw_settings.is_empty() {
        Setting::set(&state.db, "gym_name", "HHH Gym")
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        Setting::set(&state.db, "gym_description", "Your local fitness center")
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        raw_settings = Setting::list_all(&state.db)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
    }

    let settings = raw_settings
        .into_iter()
        .map(|s| {
            let description = description_for_key(&s.key);
            SettingDisplay {
                key: s.key,
                value: s.value,
                description,
            }
        })
        .collect();

    Ok(SettingsTemplate {
        settings,
        user: Some(current_user),
        flash,
    })
}

async fn save_settings(
    State(state): State<AppState>,
    RequireAdmin(_current_user): RequireAdmin,
    cookies: Cookies,
    Form(form): Form<SettingsForm>,
) -> Result<impl IntoResponse, AppError> {
    for (key, value) in form.keys.iter().zip(form.values.iter()) {
        Setting::set(&state.db, key, value)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
    }

    set_flash(&cookies, "Settings saved successfully");
    Ok(Redirect::to("/admin/settings"))
}
