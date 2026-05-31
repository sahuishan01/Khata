pub mod handlers;
pub mod middleware;
pub mod models;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
