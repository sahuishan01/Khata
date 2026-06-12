pub mod handlers;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_accounts).post(handlers::create_account))
        .route("/:id", delete(handlers::delete_account))
}
