pub mod detect;
pub mod fingerprint;
pub mod handlers;
pub mod models;
pub mod normalize;
pub mod parse;
pub mod profiles;
pub mod store;

use axum::Router;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
