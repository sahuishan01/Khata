pub mod claude_cli;
pub mod handlers;
pub mod models;
pub mod sql_validator;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
