use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth, auth::middleware::CurrentUser, error::AppError, AppState};

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
    auth::verify_ownership(&state.db, budget_id, user_id, auth::OwnedTable::Budgets).await?;

    sqlx::query("DELETE FROM budgets WHERE id = $1")
        .bind(budget_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete budget".into()))?;
    Ok(Json(serde_json::json!({ "message": "Budget deleted" })))
}

pub async fn budget_status(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<BudgetStatus>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let budgets = sqlx::query_as::<_, Budget>(
        "SELECT id, user_id, category, monthly_limit::float8 FROM budgets WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch budgets".into()))?;

    let mut result = Vec::new();
    for b in &budgets {
        let spent: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(amount), 0)::float8 FROM transactions
               WHERE user_id = $1 AND category = $2 AND direction = 'debit'
                 AND NOT deleted AND NOT is_transfer
               AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
                 AND value_date >= date_trunc('month', CURRENT_DATE)
                 AND value_date < date_trunc('month', CURRENT_DATE) + INTERVAL '1 month'"#,
        )
        .bind(user_id)
        .bind(&b.category)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AppError::BadRequest("Failed to query spend".into()))?;

        result.push(BudgetStatus {
            category: b.category.clone(),
            monthly_limit: b.monthly_limit,
            spent: spent.0,
            pct: if b.monthly_limit > 0.0 { (spent.0 / b.monthly_limit) * 100.0 } else { 0.0 },
        });
    }
    tx.commit().await?;
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{app_state, seed_txn, seed_user};
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn budget_status_counts_this_months_spend(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let today = chrono::Utc::now().date_naive();
        seed_txn(&pool, uid, "HDFC", "SWIGGY", "debit", 400.0, today, "Food & Dining").await;
        seed_txn(&pool, uid, "HDFC", "ZOMATO", "debit", 100.0, today, "Food & Dining").await;
        sqlx::query("INSERT INTO budgets (user_id, category, monthly_limit) VALUES ($1,'Food & Dining',1000)")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();

        let Json(status) = budget_status(State(app_state(&pool)), CurrentUser(uid)).await.expect("budget_status");
        assert_eq!(status.len(), 1);
        assert!((status[0].spent - 500.0).abs() < 0.001, "spent = {}", status[0].spent);
        assert!((status[0].pct - 50.0).abs() < 0.001);
    }
}
