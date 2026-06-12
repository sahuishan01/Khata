pub mod handlers;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_budgets).post(handlers::create_budget))
        .route("/status", get(handlers::budget_status))
        .route("/:id", delete(handlers::delete_budget))
}
