pub mod handlers;
pub mod models;

use axum::{routing::{get, put}, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_txns))
        .route("/dashboard", get(handlers::get_dashboard))
        .route("/analysis", get(handlers::get_analysis))
        .route("/categories", get(handlers::list_categories))
        .route("/:id/category", put(handlers::update_category))
}
