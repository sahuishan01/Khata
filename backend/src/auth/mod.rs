pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;

pub async fn verify_ownership(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Uuid,
    table: &str,
    id_column: &str,
) -> Result<(), AppError> {
    let sql = format!("SELECT user_id FROM {table} WHERE {id_column} = $1");
    let owner: Option<(Uuid,)> = sqlx::query_as(&sql)
        .bind(resource_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)?;

    match owner {
        Some((uid,)) if uid == user_id => Ok(()),
        Some(_) => Err(AppError::Forbidden),
        None => Err(AppError::NotFound),
    }
}

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
