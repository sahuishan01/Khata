use axum::{
    routing::{delete, get, post},
    Router,
};
use crate::AppState;

pub mod handlers;
pub mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_subscriptions_handler))
        .route("/", post(handlers::create_subscription_handler))
        .route("/:id", delete(handlers::delete_subscription_handler))
}
