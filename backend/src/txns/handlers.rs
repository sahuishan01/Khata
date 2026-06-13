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
    let search = params.search.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| format!("%{}%", s.to_uppercase()));
    let bank_filter = params.bank.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());

    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let (total,): (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM transactions
           WHERE user_id = $1
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)
             AND ($5::text IS NULL OR UPPER(description) LIKE $5 OR UPPER(bank_ref) LIKE $5)
             AND ($6::text IS NULL OR bank = $6)"#,
    )
    .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to).bind(&search).bind(&bank_filter)
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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes
           FROM transactions
           WHERE user_id = $1
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)
             AND ($5::text IS NULL OR UPPER(description) LIKE $5 OR UPPER(bank_ref) LIKE $5)
             AND ($6::text IS NULL OR bank = $6)
           ORDER BY {sort_col} {sort_dir}{secondary}
           LIMIT $7 OFFSET $8"#
    );
    let data = sqlx::query_as::<_, TxnRow>(&sql)
        .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to).bind(&search).bind(&bank_filter)
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
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND category IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8
           FROM transactions WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let monthly = sqlx::query_as::<_, MonthBucket>(
        r#"SELECT
             to_char(value_date, 'YYYY-MM') AS month,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8 AS spent,
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
            WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND category IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8,
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
            WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes
           FROM transactions
            WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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
            let result = sqlx::query(
                "UPDATE transactions SET category = $1 WHERE id = $2 AND user_id = $3"
            )
            .bind(&category).bind(txn_id).bind(user_id)
            .execute(&mut *tx).await?.rows_affected();

            // Create a rule for this payee
            if result > 0 {
                let desc: Option<(String,)> = sqlx::query_as(
                    "SELECT description FROM transactions WHERE id = $1 AND user_id = $2"
                )
                .bind(txn_id).bind(user_id)
                .fetch_optional(&mut *tx).await?;

                if let Some((desc_text,)) = desc {
                    let words: Vec<&str> = desc_text.split_whitespace().collect();
                    let pattern = if words.len() >= 3 {
                        format!("{} {}", words[0], words[1])
                    } else {
                        desc_text.clone()
                    };
                    let _ = sqlx::query(
                        "INSERT INTO category_rules (user_id, pattern, category) VALUES ($1, UPPER($2), $3) ON CONFLICT DO NOTHING"
                    )
                    .bind(user_id).bind(&pattern).bind(&category)
                    .execute(&mut *tx).await;
                    // Apply rule to all matching transactions
                    let like = format!("%{}%", pattern.to_uppercase());
                    let _ = sqlx::query(
                        "UPDATE transactions SET category = $1 WHERE user_id = $2 AND UPPER(description) LIKE $3 AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $2 AND txn_type = 'investment')"
                    )
                    .bind(&category).bind(user_id).bind(&like)
                    .execute(&mut *tx).await;
                }
            }
            result
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

pub async fn get_txn(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
) -> Result<Json<TxnRow>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let row = sqlx::query_as::<_, TxnRow>(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes
           FROM transactions WHERE id = $1 AND user_id = $2"#,
    )
    .bind(txn_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch transaction".into()))?;

    tx.commit().await?;

    match row {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound),
    }
}

pub async fn update_notes(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<UpdateNotesReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let result = sqlx::query(
        "UPDATE transactions SET notes = $1 WHERE id = $2 AND user_id = $3"
    )
    .bind(&req.notes)
    .bind(txn_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to update notes".into()))?;

    tx.commit().await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "message": "Notes updated" })))
}

use chrono::NaiveDate;

pub async fn create_txn(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateTxnReq>,
) -> Result<Json<TxnRow>, AppError> {
    if req.description.trim().is_empty() {
        return Err(AppError::BadRequest("Description is required".into()));
    }
    if req.direction != "debit" && req.direction != "credit" {
        return Err(AppError::BadRequest("Direction must be debit or credit".into()));
    }

    let txn_date = NaiveDate::parse_from_str(&req.txn_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid txn_date format (use YYYY-MM-DD)".into()))?;
    let value_date = NaiveDate::parse_from_str(&req.value_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid value_date format (use YYYY-MM-DD)".into()))?;

    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let row = sqlx::query_as::<_, TxnRow>(
        r#"INSERT INTO transactions
           (user_id, bank, account_label, txn_date, value_date, description, raw_description,
            amount, direction, category, bank_ref, notes, fingerprint)
           VALUES ($1, 'Manual', 'Manual', $2, $3, $4, $4, $5, $6, $7, $8, $9, gen_random_uuid()::text)
           RETURNING id, txn_date, value_date, description,
                     amount::float8, direction, balance::float8, bank, bank_ref, category,
                     is_transfer, notes"#,
    )
    .bind(user_id)
    .bind(txn_date)
    .bind(value_date)
    .bind(req.description.trim())
    .bind(req.amount)
    .bind(&req.direction)
    .bind(req.category.trim())
    .bind(req.bank_ref)
    .bind(req.notes.unwrap_or_default())
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to create transaction".into()))?;

    tx.commit().await?;
    Ok(Json(row))
}

pub async fn get_account_balances(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<AccountBalance>>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let rows = sqlx::query_as::<_, AccountBalance>(
        r#"SELECT bank, account_label,
                  COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8 AS total_spent,
                  COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8 AS total_earned,
                  COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8
                    - COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8 AS balance,
                  COUNT(*)::bigint AS txn_count
           FROM transactions WHERE user_id = $1
           GROUP BY bank, account_label ORDER BY bank"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch balances".into()))?;

    tx.commit().await?;
    Ok(Json(rows))
}

pub async fn get_recurring(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<RecurringTxn>>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let rows = sqlx::query_as::<_, (String, f64, i64, String)>(
        r#"SELECT description,
                  (SUM(amount) / COUNT(*))::float8 AS avg_amount,
                  COUNT(DISTINCT to_char(value_date, 'YYYY-MM'))::bigint AS months_active,
                  category
           FROM transactions
            WHERE user_id = $1 AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
           GROUP BY description, category
           HAVING COUNT(DISTINCT to_char(value_date, 'YYYY-MM')) >= 2
           ORDER BY months_active DESC
           LIMIT 20"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to detect recurring".into()))?;

    tx.commit().await?;

    let recurring: Vec<RecurringTxn> = rows
        .into_iter()
        .map(|(desc, amount, months, cat)| RecurringTxn {
            description: desc,
            amount,
            frequency: if months >= 3 { "monthly".to_string() } else { "occasional".to_string() },
            months,
            category: cat,
        })
        .collect();

    Ok(Json(recurring))
}
