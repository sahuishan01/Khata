pub mod handlers;
pub mod models;

use axum::{routing::{get, patch, put}, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_txns).post(handlers::create_txn))
        .route("/balances", get(handlers::get_account_balances))
        .route("/recurring", get(handlers::get_recurring))
        .route("/dashboard", get(handlers::get_dashboard))
        .route("/analysis", get(handlers::get_analysis))
        .route("/categories", get(handlers::list_categories))
        .route("/:id", get(handlers::get_txn))
        .route("/:id/category", put(handlers::update_category))
        .route("/:id/transfer", patch(handlers::toggle_transfer))
        .route("/:id/notes", patch(handlers::update_notes))
}
