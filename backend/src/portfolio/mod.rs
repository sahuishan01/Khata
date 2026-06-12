pub mod handlers;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/snapshot", get(handlers::net_worth_snapshot))
        .route("/trend", get(handlers::net_worth_trend))
        .route("/assets", get(handlers::list_assets).post(handlers::create_asset))
        .route("/assets/:id", delete(handlers::delete_asset))
        .route("/liabilities", get(handlers::list_liabilities).post(handlers::create_liability))
        .route("/liabilities/:id", delete(handlers::delete_liability))
}
