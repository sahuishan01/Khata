use axum::{
    routing::{get, post, put},
    Router,
};
use crate::AppState;

pub mod handlers;
pub mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(handlers::get_settings_handler))
        .route("/settings", put(handlers::update_settings_handler))
        .route("/consent/init", post(handlers::init_consent_handler))
        .route("/consent/list", get(handlers::list_consents_handler))
        .route("/fetch", post(handlers::manual_fetch_handler))
}
