pub mod handlers;
pub mod models;

use axum::{routing::get, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_txns))
        .route("/dashboard", get(handlers::get_dashboard))
        .route("/analysis", get(handlers::get_analysis))
}
