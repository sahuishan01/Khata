use axum::{
    routing::get,
    Router,
};
use crate::AppState;

pub mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/export/csv", get(handlers::export_csv_handler))
        .route("/tax-summary", get(handlers::tax_summary_handler))
}
