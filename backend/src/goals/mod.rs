use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use crate::AppState;

pub mod handlers;
pub mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_goals_handler))
        .route("/", post(handlers::create_goal_handler))
        .route("/:id/progress", patch(handlers::update_goal_progress_handler))
        .route("/:id", delete(handlers::delete_goal_handler))
}
