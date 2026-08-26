pub mod categorize;
pub mod crypto;
pub mod detect;
pub mod email;
pub mod fingerprint;
pub mod handlers;
pub mod models;
pub mod normalize;
pub mod parse;
pub mod profiles;
pub mod store;

use axum::{routing::delete, routing::get, routing::post, routing::put, Router};
use tower_http::limit::RequestBodyLimitLayer;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(handlers::upload_handler))
        .route("/clear", delete(handlers::clear_all_data_handler))
        .route("/email/config", get(email::get_email_config_handler))
        .route("/email/config", put(email::save_email_config_handler))
        .route("/email/config", delete(email::delete_email_config_handler))
        .route("/email/sync", post(email::trigger_email_sync_handler))
        .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024)) // 12 MB
}
