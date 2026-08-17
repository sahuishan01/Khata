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
use tower_http::limit::RequestBodyLimitLayer;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(handlers::upload_handler))
        .route("/clear", delete(handlers::clear_all_data_handler))
        .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024)) // 12 MB
}
