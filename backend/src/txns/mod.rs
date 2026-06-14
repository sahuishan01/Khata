pub mod handlers;
pub mod models;

use axum::{routing::{get, patch, post, put}, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_txns).post(handlers::create_txn))
        .route("/balances", get(handlers::get_account_balances))
        .route("/recurring", get(handlers::get_recurring))
        .route("/sync", post(handlers::sync_txns))
        .route("/dashboard", get(handlers::get_dashboard))
        .route("/analysis", get(handlers::get_analysis))
        .route("/analytics/explore", get(handlers::get_analytics_explore))
        .route("/analytics/detail", get(handlers::get_analytics_detail))
        .route("/analytics/highlights", get(handlers::get_analytics_highlights))
        .route("/categories", get(handlers::list_categories))
        .route("/:id", get(handlers::get_txn))
        .route("/:id/category", put(handlers::update_category))
        .route("/:id/transfer", patch(handlers::toggle_transfer))
        .route("/:id/notes", patch(handlers::update_notes))
}
