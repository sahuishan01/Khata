pub mod handlers;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_rules).post(handlers::create_rule))
        .route("/apply", post(handlers::apply_rules))
        .route("/:id", delete(handlers::delete_rule))
}
