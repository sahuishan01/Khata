use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

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

    let category_filter = params.category.as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let date_from = params.from;
    let date_to   = params.to;

    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let (total,): (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM transactions
           WHERE user_id = $1
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)"#,
    )
    .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to)
    .fetch_one(&mut *tx).await?;

    // Whitelist-validated sort — safe to interpolate
    let sort_col = match params.sort_by.as_deref().unwrap_or("date") {
        "amount"      => "amount",
        "description" => "description",
        "category"    => "category",
        _             => "value_date",
    };
    let sort_dir = if params.sort_dir.as_deref() == Some("asc") { "ASC" } else { "DESC" };
    let secondary = if sort_col == "value_date" {
        format!(", created_at {sort_dir}")
    } else {
        String::new()
    };

    let sql = format!(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, is_investment
           FROM transactions
           WHERE user_id = $1
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)
           ORDER BY {sort_col} {sort_dir}{secondary}
           LIMIT $5 OFFSET $6"#
    );
    let data = sqlx::query_as::<_, TxnRow>(&sql)
        .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to)
        .bind(per_page).bind(offset)
        .fetch_all(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(TxnListResponse { data, total, page, per_page }))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<DashboardStats>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let (total_spent, total_earned, total_invested): (f64, f64, f64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND NOT is_investment),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND is_investment), 0)::float8
           FROM transactions WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let monthly = sqlx::query_as::<_, MonthBucket>(
        r#"SELECT
             to_char(value_date, 'YYYY-MM') AS month,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND NOT is_investment),  0)::float8 AS spent,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8 AS earned
           FROM transactions WHERE user_id = $1
           GROUP BY month ORDER BY month DESC LIMIT 12"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    let top_debits = sqlx::query_as::<_, TopMerchant>(
        r#"SELECT description, SUM(amount)::float8 AS total
           FROM transactions
           WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND NOT is_investment
           GROUP BY description ORDER BY total DESC LIMIT 10"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(DashboardStats {
        total_spent,
        total_earned,
        total_invested,
        net: total_earned - total_spent,
        monthly,
        top_debits,
    }))
}

pub async fn get_analysis(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<AnalysisStats>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    // Total spent/earned/invested for savings rate
    let (total_spent, total_earned, total_invested, total_txns): (f64, f64, f64, i64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND NOT is_investment),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND is_investment), 0)::float8,
             COUNT(*)::bigint
           FROM transactions WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    // Category breakdown (debits only — spending categories)
    let cat_rows: Vec<(String, f64, i64)> = sqlx::query_as(
        r#"SELECT category, SUM(amount)::float8, COUNT(*)::bigint
           FROM transactions
           WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND NOT is_investment
           GROUP BY category
           ORDER BY SUM(amount) DESC"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    let category_breakdown = cat_rows
        .into_iter()
        .map(|(category, amount, txn_count)| CategoryBucket {
            category,
            amount,
            txn_count,
            pct: if total_spent > 0.0 { amount / total_spent * 100.0 } else { 0.0 },
        })
        .collect();

    // Average daily spend (days with at least one debit)
    let (active_days,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT value_date)::bigint FROM transactions WHERE user_id = $1 AND direction='debit' AND NOT is_transfer"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let avg_daily_spend = if active_days > 0 {
        total_spent / active_days as f64
    } else {
        0.0
    };

    // Month comparison: current calendar month vs previous
    let (this_month, last_month): (f64, f64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (
               WHERE value_date >= date_trunc('month', CURRENT_DATE)
                 AND NOT is_transfer
             ), 0)::float8,
             COALESCE(SUM(amount) FILTER (
               WHERE value_date >= date_trunc('month', CURRENT_DATE) - INTERVAL '1 month'
                 AND value_date  < date_trunc('month', CURRENT_DATE)
                 AND NOT is_transfer
             ), 0)::float8
           FROM transactions WHERE user_id = $1 AND direction = 'debit'"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let change_pct = if last_month > 0.0 {
        (this_month - last_month) / last_month * 100.0
    } else {
        0.0
    };

    // Largest single expense
    let largest = sqlx::query_as::<_, TxnRow>(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, is_investment
           FROM transactions
           WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND NOT is_investment
           ORDER BY amount DESC LIMIT 1"#,
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    let savings_rate_pct = if total_earned > 0.0 {
        (total_earned - total_spent).max(0.0) / total_earned * 100.0
    } else {
        0.0
    };

    Ok(Json(AnalysisStats {
        category_breakdown,
        savings_rate_pct,
        avg_daily_spend,
        month_comparison: MonthComparison { this_month, last_month, change_pct },
        largest_expense: largest,
        total_transactions: total_txns,
        total_invested,
    }))
}

// ── Category management ───────────────────────────────────────────────────────

/// Return all distinct categories for this user (used to populate dropdown).
pub async fn list_categories(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<String>>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT category FROM transactions WHERE user_id = $1 ORDER BY category"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(rows.into_iter().map(|(c,)| c).collect()))
}

/// Update a transaction's category, optionally applying to similar ones.
pub async fn update_category(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<UpdateCategoryReq>,
) -> Result<Json<UpdateCategoryResponse>, AppError> {
    let category = req.category.trim().to_string();
    if category.is_empty() {
        return Err(AppError::BadRequest("category cannot be empty".into()));
    }

    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let updated = match req.scope {
        UpdateScope::Single => {
            sqlx::query(
                "UPDATE transactions SET category = $1 WHERE id = $2 AND user_id = $3"
            )
            .bind(&category).bind(txn_id).bind(user_id)
            .execute(&mut *tx).await?.rows_affected()
        }

        UpdateScope::SameDescription => {
            // Get the description of this transaction first
            let row: Option<(String,)> = sqlx::query_as(
                "SELECT description FROM transactions WHERE id = $1 AND user_id = $2"
            )
            .bind(txn_id).bind(user_id)
            .fetch_optional(&mut *tx).await?;

            let desc = row
                .ok_or_else(|| AppError::NotFound)?
                .0;

            sqlx::query(
                "UPDATE transactions SET category = $1 WHERE user_id = $2 AND description = $3"
            )
            .bind(&category).bind(user_id).bind(&desc)
            .execute(&mut *tx).await?.rows_affected()
        }

        UpdateScope::Contains => {
            let kw = req.keyword
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("keyword required for 'contains' scope".into()))?;

            // Parameterised LIKE — safe
            let pattern = format!("%{kw}%");
            sqlx::query(
                "UPDATE transactions SET category = $1 WHERE user_id = $2 AND upper(description) LIKE upper($3)"
            )
            .bind(&category).bind(user_id).bind(&pattern)
            .execute(&mut *tx).await?.rows_affected()
        }
    };

    tx.commit().await?;
    Ok(Json(UpdateCategoryResponse { updated }))
}

#[derive(Deserialize)]
pub struct ToggleTransferReq {
    pub is_transfer: bool,
}

pub async fn toggle_transfer(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<ToggleTransferReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let result = sqlx::query(
        "UPDATE transactions SET is_transfer = $1 WHERE id = $2 AND user_id = $3"
    )
    .bind(req.is_transfer)
    .bind(txn_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to update".into()))?;

    tx.commit().await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "message": "Transfer status updated" })))
}

#[derive(Deserialize)]
pub struct ToggleInvestmentReq {
    pub is_investment: bool,
}

pub async fn toggle_investment(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<ToggleInvestmentReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let result = sqlx::query(
        "UPDATE transactions SET is_investment = $1 WHERE id = $2 AND user_id = $3"
    )
    .bind(req.is_investment)
    .bind(txn_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to update".into()))?;

    tx.commit().await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "message": "Investment status updated" })))
}
