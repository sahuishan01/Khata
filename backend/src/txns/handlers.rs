use axum::{
    extract::{Query, State},
    Json,
};

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_txns(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<ListParams>,
) -> Result<Json<TxnListResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    // Set RLS context on the main pool connection
    // We use a transaction to scope SET LOCAL
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let data = sqlx::query_as::<_, TxnRow>(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref
           FROM transactions
           WHERE user_id = $1
           ORDER BY value_date DESC, created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(TxnListResponse {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<DashboardStats>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let (total_spent, total_earned): (f64, f64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (WHERE direction='debit'),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit'), 0)::float8
           FROM transactions WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let monthly = sqlx::query_as::<_, MonthBucket>(
        r#"SELECT
             to_char(value_date, 'YYYY-MM') AS month,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit'),  0)::float8 AS spent,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit'), 0)::float8 AS earned
           FROM transactions WHERE user_id = $1
           GROUP BY month ORDER BY month DESC LIMIT 12"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    let top_debits = sqlx::query_as::<_, TopMerchant>(
        r#"SELECT description, SUM(amount)::float8 AS total
           FROM transactions
           WHERE user_id = $1 AND direction = 'debit'
           GROUP BY description ORDER BY total DESC LIMIT 10"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(DashboardStats {
        total_spent,
        total_earned,
        net: total_earned - total_spent,
        monthly,
        top_debits,
    }))
}
