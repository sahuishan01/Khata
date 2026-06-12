pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register_handler))
        .route("/login", post(handlers::login_handler))
        .route("/setup", post(handlers::setup_handler))
        .route("/setup-status", get(handlers::setup_status_handler))
        .route("/me", get(handlers::me_handler))
        .route("/users", get(handlers::list_users_handler).post(handlers::admin_create_user_handler))
        .route("/users/:id", delete(handlers::delete_user_handler))
        .route("/reset-password", post(handlers::reset_password_handler))
        .route("/email", post(handlers::update_email_handler))
}
