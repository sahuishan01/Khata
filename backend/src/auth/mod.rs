pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{routing::delete, routing::get, routing::post, Router};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppState;
use crate::error::AppError;

/// Table that supports ownership verification via `user_id` column.
pub enum OwnedTable {
    UserAccounts,
    Budgets,
    CategoryRules,
    PortfolioAssets,
    PortfolioLiabilities,
    Categories,
}

impl OwnedTable {
    fn table_name(&self) -> &'static str {
        match self {
            Self::UserAccounts => "user_accounts",
            Self::Budgets => "budgets",
            Self::CategoryRules => "category_rules",
            Self::PortfolioAssets => "portfolio_assets",
            Self::PortfolioLiabilities => "portfolio_liabilities",
            Self::Categories => "categories",
        }
    }
}

pub async fn verify_ownership(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Uuid,
    table: OwnedTable,
) -> Result<(), AppError> {
    let sql = format!("SELECT user_id FROM {} WHERE id = $1", table.table_name());
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
        .route("/logout", post(handlers::logout_handler))
        .route("/email", post(handlers::update_email_handler))
}
