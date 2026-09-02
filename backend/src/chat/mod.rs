pub mod claude_cli;
pub mod handlers;
pub mod models;
pub mod predefined;
pub mod sql_validator;

use axum::{routing::{get, post}, Router};
use tower_http::limit::RequestBodyLimitLayer;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ask",     post(handlers::ask_handler))
        .route("/history", get(handlers::history_handler))
        .layer(RequestBodyLimitLayer::new(64 * 1024)) // 64 KB — chat bodies are tiny
}
