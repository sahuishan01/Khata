pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register_handler))
        .route("/login", post(handlers::login_handler))
}
