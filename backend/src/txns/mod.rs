pub mod handlers;
pub mod models;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
