use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_budgets(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<Budget>>, AppError> {
    let budgets = sqlx::query_as::<_, Budget>(
        "SELECT id, user_id, category, monthly_limit::float8 FROM budgets WHERE user_id = $1 ORDER BY category",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch budgets".into()))?;
    Ok(Json(budgets))
}

pub async fn create_budget(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateBudgetReq>,
) -> Result<Json<Budget>, AppError> {
    if req.category.trim().is_empty() || req.monthly_limit <= 0.0 {
        return Err(AppError::BadRequest("Valid category and monthly_limit required".into()));
    }

    let budget = sqlx::query_as::<_, Budget>(
        "INSERT INTO budgets (user_id, category, monthly_limit) VALUES ($1, $2, $3)
         RETURNING id, user_id, category, monthly_limit::float8",
    )
    .bind(user_id)
    .bind(req.category.trim())
    .bind(req.monthly_limit)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.code().as_deref() == Some("23505") {
                return AppError::Conflict("Budget for this category already exists".into());
            }
        }
        AppError::BadRequest("Failed to create budget".into())
    })?;
    Ok(Json(budget))
}

pub async fn delete_budget(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(budget_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM budgets WHERE id = $1 AND user_id = $2")
        .bind(budget_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete budget".into()))?;
    if result.rows_affected() == 0 { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "message": "Budget deleted" })))
}

pub async fn budget_status(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<BudgetStatus>>, AppError> {
    let budgets = sqlx::query_as::<_, Budget>(
        "SELECT id, user_id, category, monthly_limit::float8 FROM budgets WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch budgets".into()))?;

    let mut result = Vec::new();
    for b in &budgets {
        let spent: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(amount), 0)::float8 FROM transactions
               WHERE user_id = $1 AND category = $2 AND direction = 'debit'
                 AND NOT is_transfer AND NOT is_investment
                 AND value_date >= date_trunc('month', CURRENT_DATE)
                 AND value_date < date_trunc('month', CURRENT_DATE) + INTERVAL '1 month'"#,
        )
        .bind(user_id)
        .bind(&b.category)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to query spend".into()))?;

        result.push(BudgetStatus {
            category: b.category.clone(),
            monthly_limit: b.monthly_limit,
            spent: spent.0,
            pct: if b.monthly_limit > 0.0 { (spent.0 / b.monthly_limit) * 100.0 } else { 0.0 },
        });
    }
    Ok(Json(result))
}
