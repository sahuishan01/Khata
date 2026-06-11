pub mod categorize;
pub mod detect;
pub mod fingerprint;
pub mod handlers;
pub mod models;
pub mod normalize;
pub mod parse;
pub mod profiles;
pub mod store;

use axum::{routing::delete, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(handlers::upload_handler))
        .route("/debug-headers", post(handlers::debug_headers_handler))
        .route("/clear", delete(handlers::clear_all_data_handler))
}
