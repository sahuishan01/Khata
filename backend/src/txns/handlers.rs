use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

// A review suggestion only: repeated payments without references can be legitimate.
const MATCHING_RECORD: &str = "d.user_id = t.user_id AND d.id <> t.id AND NOT d.deleted \
    AND upper(d.bank) = upper(t.bank) AND d.amount = t.amount \
    AND d.direction = t.direction AND ((d.value_date = t.value_date AND d.description = t.description \
    AND d.bank_ref IS NOT DISTINCT FROM t.bank_ref) OR is_card_confirmation_candidate(t,d))";


pub async fn list_txns(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<ListParams>,
) -> Result<Json<TxnListResponse>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1).saturating_mul(per_page);

    let category_filter = params.category.as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let date_from = params.from;
    let date_to   = params.to;
    let search = params.search.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| format!("%{}%", s.to_uppercase()));
    let bank_filter = params.bank.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let (total,): (i64,) = sqlx::query_as(
        &format!(r#"SELECT COUNT(*) FROM transactions t
           WHERE user_id = $1 AND NOT deleted
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)
             AND ($5::text IS NULL OR UPPER(description) LIKE $5 OR UPPER(bank_ref) LIKE $5 OR UPPER(category) LIKE $5 OR UPPER(notes) LIKE $5)
             AND ($6::text IS NULL OR bank = $6)
             AND (NOT $7 OR EXISTS (SELECT 1 FROM transactions d WHERE {MATCHING_RECORD}))"#),
    )
    .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to).bind(&search).bind(&bank_filter).bind(params.duplicates_only.unwrap_or(false))
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
        String::from(", value_date DESC, created_at DESC")
    };

    let sql = format!(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version,
                  rev, base_rev, deleted, updated_at, client_id, sms_source,
                  EXISTS (SELECT 1 FROM transactions d WHERE {MATCHING_RECORD}) AS possible_duplicate
           FROM transactions t
           WHERE user_id = $1 AND NOT deleted
             AND ($2::text IS NULL OR category = $2)
             AND ($3::date IS NULL OR value_date >= $3)
             AND ($4::date IS NULL OR value_date <= $4)
             AND ($5::text IS NULL OR UPPER(description) LIKE $5 OR UPPER(bank_ref) LIKE $5 OR UPPER(category) LIKE $5 OR UPPER(notes) LIKE $5)
             AND ($6::text IS NULL OR bank = $6)
             AND (NOT $7 OR EXISTS (SELECT 1 FROM transactions d WHERE {MATCHING_RECORD}))
           ORDER BY {sort_col} {sort_dir}{secondary}, id {sort_dir}
           LIMIT $8 OFFSET $9"#
    );
    let data = sqlx::query_as::<_, TxnRow>(&sql)
        .bind(user_id).bind(&category_filter).bind(date_from).bind(date_to).bind(&search).bind(&bank_filter)
        .bind(params.duplicates_only.unwrap_or(false)).bind(per_page).bind(offset)
        .fetch_all(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(TxnListResponse { data, total, page, per_page }))
}

/// The user confirms one suggested row is a duplicate. Retain its source identity.
pub async fn exclude_duplicate(
    State(state): State<AppState>, CurrentUser(user_id): CurrentUser, Path(txn_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user_id).execute(&mut *tx).await?;
    let canonical: Option<Uuid> = sqlx::query_scalar(&format!(
        "SELECT d.id FROM transactions t JOIN transactions d ON {MATCHING_RECORD} \
         WHERE t.id=$1 AND t.user_id=$2 AND NOT t.deleted \
         ORDER BY d.created_at,d.id LIMIT 1"
    )).bind(txn_id).bind(user_id).fetch_optional(&mut *tx).await?;
    let canonical = canonical.ok_or_else(|| AppError::BadRequest("No matching transaction remains; refresh the list".into()))?;
    // Keep every older alias attached to a live root when reviewing a group of 3+.
    sqlx::query("UPDATE transactions SET duplicate_of=$1,rev=rev+1,version=version+1,updated_at=now() WHERE duplicate_of=$2 AND user_id=$3")
        .bind(canonical).bind(txn_id).bind(user_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE transactions SET duplicate_of=$1,deleted=true,duplicate_override=false,duplicate_reason='user_confirmed',rev=rev+1,version=version+1,updated_at=now() WHERE id=$2 AND user_id=$3")
        .bind(canonical).bind(txn_id).bind(user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(%user_id, %txn_id, "Excluded user-confirmed duplicate");
    Ok(Json(serde_json::json!({"message":"Duplicate excluded. The matching transaction is still included."})))
}

pub async fn restore_duplicate(
    State(state): State<AppState>, CurrentUser(user_id): CurrentUser, Path(txn_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let result = sqlx::query("UPDATE transactions SET duplicate_of=NULL,deleted=false,duplicate_override=true,duplicate_reason=NULL,rev=rev+1,version=version+1,updated_at=now() WHERE id=$1 AND user_id=$2 AND duplicate_of IS NOT NULL")
        .bind(txn_id).bind(user_id).execute(&mut *tx).await?;
    if result.rows_affected() == 0 { return Err(AppError::NotFound); }
    tx.commit().await?;
    tracing::info!(%user_id, %txn_id, "Restored excluded duplicate");
    Ok(Json(serde_json::json!({"message":"Transaction restored"})))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<DashboardStats>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let (total_spent, total_earned, total_invested): (f64, f64, f64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8
           FROM transactions WHERE user_id = $1 AND NOT deleted"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let monthly = sqlx::query_as::<_, MonthBucket>(
        r#"SELECT
             to_char(value_date, 'YYYY-MM') AS month,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8 AS spent,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8 AS earned
           FROM transactions WHERE user_id = $1 AND NOT deleted
           GROUP BY month ORDER BY month DESC LIMIT 12"#,
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    let top_debits = sqlx::query_as::<_, TopMerchant>(
        r#"SELECT description, SUM(amount)::float8 AS total
           FROM transactions
            WHERE user_id = $1 AND NOT deleted AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    // Total spent/earned/invested for savings rate
    let (total_spent, total_earned, total_invested, total_txns): (f64, f64, f64, i64) = sqlx::query_as(
        r#"SELECT
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')),  0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8,
             COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8,
             COUNT(*) FILTER (WHERE NOT is_transfer)::bigint
           FROM transactions WHERE user_id = $1 AND NOT deleted"#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    // Category breakdown (debits only — spending categories)
    let cat_rows: Vec<(String, f64, i64)> = sqlx::query_as(
        r#"SELECT category, SUM(amount)::float8, COUNT(*)::bigint
           FROM transactions
            WHERE user_id = $1 AND NOT deleted AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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
        "SELECT COUNT(DISTINCT value_date)::bigint FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer"
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
           FROM transactions WHERE user_id = $1 AND NOT deleted AND direction = 'debit'"#,
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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version,
                  rev, base_rev, deleted, updated_at, client_id, sms_source
           FROM transactions
            WHERE user_id = $1 AND NOT deleted AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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

// ── Analytics Explore ─────────────────────────────────────────────────────────

pub async fn get_analytics_explore(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let group_col = match params.group_by.as_deref() {
        Some("payee") => "description",
        Some("account") => "account_label",
        Some("week") => "to_char(value_date, 'IYYY-\"W\"IW')",
        Some("month") => "to_char(value_date, 'YYYY-MM')",
        _ => "category",
    };

    let dim_where = match params.dimension.as_deref() {
        Some("earned") => "AND direction = 'credit' AND NOT is_transfer".to_string(),
        Some("net") => "AND NOT is_transfer".to_string(),
        _ => "AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')".to_string(),
    };

    let mut wheres = vec!["user_id = $1".to_string(), "NOT deleted".to_string()];
    let mut bind_idx = 2;

    if let Some(ref from) = params.from {
        wheres.push(format!("value_date >= ${}", bind_idx));
        bind_idx += 1;
    }
    if let Some(ref to) = params.to {
        wheres.push(format!("value_date <= ${}", bind_idx));
        bind_idx += 1;
    }
    if let Some(ref cat) = params.category {
        wheres.push(format!("category = ${}", bind_idx));
        bind_idx += 1;
    }
    if let Some(ref bank) = params.bank {
        wheres.push(format!("bank = ${}", bind_idx));
        bind_idx += 1;
    }
    if let Some(ref dir) = params.direction {
        if dir == "debit" {
            wheres.push("direction = 'debit'".to_string());
        } else if dir == "credit" {
            wheres.push("direction = 'credit'".to_string());
        }
    }

    let where_clause = wheres.join(" AND ");

    let sql = format!(
        "SELECT {} AS label, SUM(amount)::float8 AS value \
         FROM transactions WHERE {} {} \
         GROUP BY label ORDER BY value DESC LIMIT 20",
        group_col, where_clause, dim_where
    );

    let mut q = sqlx::query_as::<_, (String, f64)>(&sql);
    q = q.bind(user_id);
    if let Some(ref from) = params.from {
        q = q.bind(from);
    }
    if let Some(ref to) = params.to {
        q = q.bind(to);
    }
    if let Some(ref cat) = params.category {
        q = q.bind(cat);
    }
    if let Some(ref bank) = params.bank {
        q = q.bind(bank);
    }

    let rows: Vec<(String, f64)> = q
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| { tracing::error!("Analytics query error: {e}"); AppError::BadRequest("Query failed".into()) })?;

    let total: f64 = rows.iter().map(|(_, v)| v).sum();
    let labels: Vec<String> = rows.iter().map(|(l, _)| l.clone()).collect();
    let values: Vec<f64> = rows.iter().map(|(_, v)| *v).collect();

    let comparison_values = None;

    let mut insights = vec![];
    if let Some((top_label, top_val)) = rows.first() {
        if total > 0.0 {
            let pct = top_val / total * 100.0;
            insights.push(format!(
                "{}% of spending goes to {}",
                pct.round(),
                top_label
            ));
        }
    }

    tx.commit().await?;

    Ok(Json(AnalyticsResponse {
        series: AnalyticsSeries {
            labels,
            values,
            comparison_values,
            total,
        },
        insights,
    }))
}

// ── Analytics Detail ──────────────────────────────────────────────────────────

pub async fn get_analytics_detail(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<DetailQuery>,
) -> Result<Json<DetailResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let mut wheres = vec!["user_id = $1".to_string(), "NOT deleted".to_string()];
    let mut bind_idx = 2;
    if let Some(ref cat) = params.category {
        wheres.push(format!("category = ${}", bind_idx)); bind_idx += 1;
    }
    if let Some(ref m) = params.month {
        wheres.push(format!("to_char(value_date, 'YYYY-MM') = ${}", bind_idx)); bind_idx += 1;
    }
    if let Some(ref p) = params.payee {
        wheres.push(format!("description = ${}", bind_idx)); bind_idx += 1;
    }
    let w = wheres.join(" AND ");

    let sql = format!("SELECT COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0)::float8, COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer),0)::float8, COUNT(*) FILTER (WHERE NOT is_transfer)::bigint FROM transactions WHERE {}", w);
    let mut q = sqlx::query_as::<_, (f64, f64, i64)>(&sql).bind(user_id);
    if let Some(ref v) = params.category { q = q.bind(v); }
    if let Some(ref v) = params.month { q = q.bind(v); }
    if let Some(ref v) = params.payee { q = q.bind(v); }
    let (total_spent, total_earned, txn_count): (f64, f64, i64) = q.fetch_one(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let sql = format!("SELECT to_char(value_date, 'YYYY-MM') AS month, COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0)::float8 AS spent, COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer),0)::float8 AS earned FROM transactions WHERE {} GROUP BY month ORDER BY month DESC LIMIT 12", w);
    let mut q = sqlx::query_as::<_, MonthBucket>(&sql).bind(user_id);
    if let Some(ref v) = params.category { q = q.bind(v); }
    if let Some(ref v) = params.month { q = q.bind(v); }
    if let Some(ref v) = params.payee { q = q.bind(v); }
    let trend: Vec<MonthBucket> = q.fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let sql = format!("SELECT description, SUM(amount)::float8 AS total FROM transactions WHERE {} AND direction='debit' AND NOT is_transfer GROUP BY description ORDER BY total DESC LIMIT 10", w);
    let mut q = sqlx::query_as::<_, TopMerchant>(&sql).bind(user_id);
    if let Some(ref v) = params.category { q = q.bind(v); }
    if let Some(ref v) = params.month { q = q.bind(v); }
    if let Some(ref v) = params.payee { q = q.bind(v); }
    let top_payees: Vec<TopMerchant> = q.fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let sql = format!("SELECT category, SUM(amount)::float8 AS amount, COUNT(*)::bigint AS txn_count, 0.0::float8 AS pct FROM transactions WHERE {} AND direction='debit' AND NOT is_transfer GROUP BY category ORDER BY amount DESC LIMIT 10", w);
    let mut q = sqlx::query_as::<_, CategoryBucket>(&sql).bind(user_id);
    if let Some(ref v) = params.category { q = q.bind(v); }
    if let Some(ref v) = params.month { q = q.bind(v); }
    if let Some(ref v) = params.payee { q = q.bind(v); }
    let top_categories: Vec<CategoryBucket> = q.fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    tx.commit().await?;
    Ok(Json(DetailResponse { total_spent, total_earned, txn_count, trend, top_payees, top_categories }))
}

pub async fn get_analytics_highlights(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<HighlightsResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let highest_earning_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='credit' AND NOT is_transfer GROUP BY month ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let highest_spending_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type='investment') GROUP BY month ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let biggest_expense: Option<TxnRow> = sqlx::query_as(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version, rev, base_rev, deleted, updated_at, client_id, sms_source FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer ORDER BY amount DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let biggest_income: Option<TxnRow> = sqlx::query_as(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version, rev, base_rev, deleted, updated_at, client_id, sms_source FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='credit' AND NOT is_transfer ORDER BY amount DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let top_payee: Option<(String, f64)> = sqlx::query_as(
        "SELECT description, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer GROUP BY description ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let most_frequent_payee: Option<(String, i64)> = sqlx::query_as(
        "SELECT description, COUNT(*)::bigint FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer GROUP BY description ORDER BY COUNT(*) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let top_category: Option<(String, f64, i64)> = sqlx::query_as(
        "SELECT category, SUM(amount)::float8, COUNT(*)::bigint FROM transactions WHERE user_id = $1 AND NOT deleted AND direction='debit' AND NOT is_transfer GROUP BY category ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    let best_savings_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, \
         (COALESCE(SUM(amount) FILTER (WHERE direction='credit'),0) - \
          COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0))::float8 / \
         NULLIF(SUM(amount) FILTER (WHERE direction='credit'),0) * 100.0 AS savings_rate \
         FROM transactions WHERE user_id = $1 AND NOT deleted AND NOT is_transfer GROUP BY month HAVING COALESCE(SUM(amount) FILTER (WHERE direction='credit'),0) > 0 ORDER BY savings_rate DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest("Query failed".into()))?;

    tx.commit().await?;
    Ok(Json(HighlightsResponse {
        highest_earning_month: highest_earning_month.map(|(m, v)| MonthBucket { month: m, spent: 0.0, earned: v }),
        highest_spending_month: highest_spending_month.map(|(m, v)| MonthBucket { month: m, spent: v, earned: 0.0 }),
        biggest_expense,
        biggest_income,
        top_payee: top_payee.map(|(d, v)| TopMerchant { description: d, total: v }),
        most_frequent_payee,
        top_category: top_category.map(|(c, a, t)| CategoryBucket { category: c, amount: a, txn_count: t, pct: 0.0 }),
        best_savings_month: best_savings_month.map(|(m, r)| (m, r)),
    }))
}

// ── Category management ───────────────────────────────────────────────────────

/// Return all distinct categories for this user (used to populate dropdown).
pub async fn list_categories(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<String>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT category FROM transactions WHERE user_id = $1 AND NOT deleted ORDER BY category"
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let result = sqlx::query(
        "UPDATE transactions SET is_transfer = $1, transfer_override = true, version = version + 1, rev = rev + 1, updated_at = NOW() WHERE id = $2 AND user_id = $3"
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row = sqlx::query_as::<_, TxnRow>(
        r#"SELECT id, txn_date, value_date, description,
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version,
                  rev, base_rev, deleted, updated_at, client_id, sms_source
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

/// Save all editable fields atomically, preserving source identity and raw text.
pub async fn update_txn(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<UpdateTxnReq>,
) -> Result<Json<TxnRow>, AppError> {
    fn valid_money(value: f64) -> bool {
        value.is_finite() && value.abs() <= 999_999_999_999.99
            && (value - (value * 100.0).round() / 100.0).abs() <= f64::EPSILON * value.abs().max(1.0)
    }
    if req.base_rev < 0 || req.description.trim().is_empty() || req.description.chars().count() > 16_000 {
        return Err(AppError::BadRequest("Description must contain 1–16000 characters".into()));
    }
    if !valid_money(req.amount) || req.amount <= 0.0 || req.balance.is_some_and(|v| !valid_money(v)) {
        return Err(AppError::BadRequest("Enter a positive amount and valid balance with at most two decimal places".into()));
    }
    if !matches!(req.direction.as_str(), "debit" | "credit") {
        return Err(AppError::BadRequest("Direction must be debit or credit".into()));
    }
    if req.category.trim().is_empty() || req.category.chars().count() > 100
        || req.bank.trim().is_empty() || req.bank.chars().count() > 100
        || req.bank_ref.as_ref().is_some_and(|r| r.chars().count() > 500)
        || req.notes.chars().count() > 16_000 {
        return Err(AppError::BadRequest("Check category, bank, reference and notes lengths".into()));
    }
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    // Use the same tenant lock order as sync and the identity triggers.
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user_id).execute(&mut *tx).await?;
    let revision: Option<i64> = sqlx::query_scalar(
        "SELECT rev FROM transactions WHERE id=$1 AND user_id=$2 AND NOT deleted FOR UPDATE"
    ).bind(txn_id).bind(user_id).fetch_optional(&mut *tx).await?;
    let revision = revision.ok_or(AppError::NotFound)?;
    if revision != req.base_rev {
        return Err(AppError::Conflict("This transaction changed. Reopen it to review the latest values before saving.".into()));
    }
    // Mark SMS provenance first. The legacy UPDATE OF sms_source trigger keeps
    // old transfer choices during evidence repairs, so the user-field update
    // below deliberately leaves that column out. Both writes commit atomically.
    sqlx::query(
        "UPDATE transactions SET sms_source=CASE WHEN sms_source IS NOT NULL \
             THEN sms_source || '{\"user_edited\":true}'::jsonb ELSE jsonb_build_object('parser_version',0, \
                 'date_source','queue','parsed_date',txn_date,'user_edited',true) END \
         WHERE id=$1 AND user_id=$2 AND (sms_source IS NOT NULL OR notes LIKE 'Auto-captured via SMS%')"
    ).bind(txn_id).bind(user_id).execute(&mut *tx).await?;
    sqlx::query(
        "UPDATE transactions SET description=$1, amount=$2, direction=$3, category=$4, \
         txn_date=$5, value_date=$6, bank=$7, bank_ref=$8, balance=$9, is_transfer=$10, notes=$11, \
         transfer_override=true, duplicate_override=true, version=version+1, base_rev=rev, rev=rev+1, updated_at=NOW() \
         WHERE id=$12 AND user_id=$13"
    ).bind(&req.description).bind(req.amount).bind(&req.direction).bind(req.category.trim())
        .bind(req.txn_date).bind(req.value_date).bind(req.bank.trim()).bind(&req.bank_ref)
        .bind(req.balance).bind(req.is_transfer).bind(&req.notes).bind(txn_id).bind(user_id)
        .execute(&mut *tx).await?;
    // Return the committed canonical revision after deferred reconciliation.
    sqlx::query("SET CONSTRAINTS ALL IMMEDIATE").execute(&mut *tx).await?;
    let row = sqlx::query_as::<_, TxnRow>(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, \
         bank, bank_ref, category, is_transfer, notes, version, rev, base_rev, deleted, updated_at, client_id, sms_source \
         FROM transactions WHERE id=$1 AND user_id=$2"
    ).bind(txn_id).bind(user_id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(row))
}

/// Edit the displayed title without changing the import identity or source text.
pub async fn update_description(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<UpdateDescriptionReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let description = req.description.trim();
    if description.is_empty() || description.chars().count() > 500 {
        return Err(AppError::BadRequest("Title must contain 1–500 characters".into()));
    }
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let result = sqlx::query(
        "UPDATE transactions SET description = $1, version = version + 1, rev = rev + 1, updated_at = NOW() \
         WHERE id = $2 AND user_id = $3 AND NOT deleted"
    )
    .bind(description).bind(txn_id).bind(user_id)
    .execute(&mut *tx).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "message": "Title updated" })))
}

pub async fn update_notes(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(txn_id): Path<Uuid>,
    Json(req): Json<UpdateNotesReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let result = sqlx::query(
        "UPDATE transactions SET notes = $1, version = version + 1 WHERE id = $2 AND user_id = $3"
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row = sqlx::query_as::<_, TxnRow>(
        r#"INSERT INTO transactions
           (user_id, bank, account_label, txn_date, value_date, description, raw_description,
            amount, direction, category, bank_ref, notes, version, fingerprint)
           VALUES ($1, 'Manual', 'Manual', $2, $3, $4, $4, $5, $6, $7, $8, $9, gen_random_uuid()::text)
            RETURNING id, txn_date, value_date, description,
                      amount::float8, direction, balance::float8, bank, bank_ref, category,
                      is_transfer, notes, version,
                      rev, base_rev, deleted, updated_at, client_id, sms_source"#,
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
    .bind(1) // version
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows = sqlx::query_as::<_, AccountBalance>(
        r#"SELECT bank, account_label,
                  COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8 AS total_spent,
                  COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8 AS total_earned,
                  COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer), 0)::float8
                    - COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')), 0)::float8 AS balance,
                  COUNT(*) FILTER (WHERE NOT is_transfer)::bigint AS txn_count
           FROM transactions WHERE user_id = $1 AND NOT deleted
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
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows = sqlx::query_as::<_, (String, f64, i64, String)>(
        r#"SELECT description,
                  (SUM(amount) / COUNT(*))::float8 AS avg_amount,
                  COUNT(DISTINCT to_char(value_date, 'YYYY-MM'))::bigint AS months_active,
                  category
           FROM transactions
            WHERE user_id = $1 AND NOT deleted AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')
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

pub async fn sync_txns(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(batch): Json<Vec<SyncTxn>>,
) -> Result<Json<SyncResult>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let mut success: Vec<String> = Vec::new();
    let mut conflicts: Vec<SyncConflict> = Vec::new();

    for item in &batch {
        let id = uuid::Uuid::parse_str(&item.id).map_err(|_| AppError::BadRequest("Invalid UUID".into()))?;

        // Get current server version
        let server: Option<(i32,)> = sqlx::query_as(
            "SELECT version FROM transactions WHERE id = $1 AND user_id = $2"
        )
        .bind(id).bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AppError::BadRequest("DB error".into()))?;

        let Some((server_version,)) = server else {
            // Transaction doesn't exist on server — skip
            success.push(item.id.clone());
            continue;
        };

        if server_version != item.version {
            // Conflict! Fetch full server txn for conflict resolution
            let server_txn = sqlx::query_as::<_, TxnRow>(
                r#"SELECT id, txn_date, value_date, description,
                          amount::float8, direction, balance::float8, bank, bank_ref, category,
                          is_transfer, notes, version,
                          rev, base_rev, deleted, updated_at, client_id, sms_source
                   FROM transactions WHERE id = $1 AND user_id = $2"#,
            )
            .bind(id).bind(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| AppError::BadRequest("DB error".into()))?
            .unwrap();

            conflicts.push(SyncConflict {
                id: item.id.clone(),
                local_version: item.version,
                server_version,
                server_txn,
            });
            continue;
        }

        // Version matches — apply updates
        if let Some(cat) = &item.category {
            sqlx::query("UPDATE transactions SET category = $1, version = version + 1 WHERE id = $2 AND user_id = $3")
                .bind(cat).bind(id).bind(user_id)
                .execute(&mut *tx).await?;
        }
        if let Some(notes) = &item.notes {
            sqlx::query("UPDATE transactions SET notes = $1, version = version + 1 WHERE id = $2 AND user_id = $3")
                .bind(notes).bind(id).bind(user_id)
                .execute(&mut *tx).await?;
        }
        if let Some(is_transfer) = item.is_transfer {
            sqlx::query("UPDATE transactions SET is_transfer = $1, transfer_override = true, version = version + 1 WHERE id = $2 AND user_id = $3")
                .bind(is_transfer).bind(id).bind(user_id)
                .execute(&mut *tx).await?;
        }

        success.push(item.id.clone());
    }

    tx.commit().await?;
    Ok(Json(SyncResult { success, conflicts }))
}

pub async fn sync_pull(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<SyncPullQuery>,
) -> Result<Json<SyncPullResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let since = params.since_rev.unwrap_or(0);
    let limit = params.limit.unwrap_or(200).clamp(1, 1000);
    let change_cursor = params.cursor_version.unwrap_or(1) >= 2;
    let cursor_column = if change_cursor { "sync_rev" } else { "rev" };

    let rows = sqlx::query_as::<_, TxnRow>(&format!(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, \
         balance::float8, bank, bank_ref, category, is_transfer, notes, version, \
         rev, base_rev, deleted, updated_at, client_id, sms_source, sync_rev \
         FROM transactions WHERE user_id = $1 AND {cursor_column} > $2 \
         ORDER BY {cursor_column} ASC LIMIT $3"
    ))
    .bind(user_id).bind(since).bind(limit)
    .fetch_all(&mut *tx).await?;

    let new_rev = rows.last().map(|r| if change_cursor { r.sync_rev } else { r.rev }).unwrap_or(since);
    let has_more = rows.len() as i64 >= limit;

    tx.commit().await?;
    Ok(Json(SyncPullResponse { txns: rows, new_rev, has_more }))
}

#[derive(sqlx::FromRow)]
struct SyncStoredTransaction {
    #[sqlx(flatten)]
    txn: TxnRow,
    transfer_override: bool,
    duplicate_of: Option<Uuid>,
}

/// A repeated upload is not an edit conflict. Only tolerate flag differences
/// backed by the identical SMS and an existing server classification.
fn is_sync_replay(op: &SyncPushOp, stored: &SyncStoredTransaction) -> bool {
    fn unchanged<T: PartialEq>(requested: &Option<T>, current: &T) -> bool {
        requested.as_ref().is_none_or(|value| value == current)
    }
    fn same_source(requested: &SmsSource, current: &SmsSource) -> bool {
        let mut requested = requested.clone();
        // Older Gson DTOs omit additive keys. The replay still needs identical
        // original provenance and financial fields, and never replaces metadata.
        requested.user_edited = current.user_edited;
        if requested.parsed_payee.is_none() {
            requested.parsed_payee = current.parsed_payee.clone();
        }
        requested == *current
    }
    let row = &stored.txn;
    let unchanged_source = op.sms_source.as_ref().is_none_or(|source|
        row.sms_source.as_ref().is_some_and(|current| same_source(source, current)));
    let same_sms = op.sms_source.as_ref().is_some_and(|source|
        source.body_hash.is_some() && source.received_at.is_some()
        && unchanged_source
        && (op.notes.as_deref().is_some_and(|notes| notes.starts_with("Auto-captured via SMS"))
            || op.description.as_deref() == Some(format!("Auto-captured via SMS ({})", row.bank).as_str()))
        && row.notes == op.notes.as_deref().unwrap_or_default());
    unchanged(&op.amount, &row.amount)
        && unchanged(&op.direction, &row.direction)
        && unchanged(&op.description, &row.description)
        && unchanged(&op.category, &row.category)
        && unchanged(&op.bank, &row.bank)
        && op.bank_ref.as_ref().is_none_or(|reference| row.bank_ref.as_ref() == Some(reference))
        && unchanged(&op.txn_date, &row.txn_date)
        && unchanged(&op.value_date, &row.value_date)
        && unchanged(&op.notes, &row.notes)
        && unchanged_source
        && (unchanged(&op.is_transfer, &row.is_transfer)
            || (same_sms && op.is_transfer == Some(false) && row.is_transfer && !stored.transfer_override))
        && (unchanged(&op.deleted, &row.deleted)
            || (same_sms && op.deleted == Some(false) && row.deleted && stored.duplicate_of.is_some()))
}

/// Deferred repayment rules can revise earlier rows in this same batch. Return
/// revisions only after those rules finish, for both upload and resolution.
async fn finish_sync_revisions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    accepted: &mut [SyncAccepted],
) -> Result<(), AppError> {
    sqlx::query("SET CONSTRAINTS ALL IMMEDIATE").execute(&mut **tx).await?;
    for acknowledgement in accepted {
        acknowledgement.server_rev = sqlx::query_scalar(
            "SELECT rev FROM transactions WHERE user_id=$1 AND client_id=$2"
        ).bind(user_id).bind(acknowledgement.client_id).fetch_one(&mut **tx).await?;
    }
    Ok(())
}

pub async fn sync_push(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(ops): Json<Vec<SyncPushOp>>,
) -> Result<Json<SyncPushResult>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    // Serialize retries before reading revisions, using the same owner lock as
    // transaction triggers. Concurrent copies of a create must see its commit.
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user_id).execute(&mut *tx).await?;

    let mut accepted = vec![];
    let mut conflicts = vec![];

    for op in ops {
        if let Some(source) = &op.sms_source { source.validate()?; }
        let server_row: Option<SyncStoredTransaction> = sqlx::query_as(
            "SELECT id, txn_date, value_date, description, amount::float8, direction, \
             balance::float8, bank, bank_ref, category, is_transfer, notes, version, \
             rev, base_rev, deleted, updated_at, client_id, sms_source, transfer_override, duplicate_of \
             FROM transactions WHERE client_id = $1 AND user_id = $2"
        )
        .bind(op.client_id).bind(user_id)
        .fetch_optional(&mut *tx).await?;

        match server_row {
            None => {
                let description = op.description.unwrap_or_default();
                let fallback_date = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(19800).unwrap()).date_naive();
                let txn_date = op.txn_date.or(op.value_date).unwrap_or(fallback_date);
                let value_date = op.value_date.unwrap_or(txn_date);
                let fingerprint = format!("device:{}:{}", user_id, op.client_id);
                let new_rev: i64 = sqlx::query_scalar(
                    "INSERT INTO transactions (user_id, client_id, description, amount, direction, category, \
                     txn_date, value_date, notes, is_transfer, deleted, bank, bank_ref, raw_description, fingerprint, rev, base_rev, sms_source) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9,''), $10, $11, $12, $13, $3, $14, 1, 1, $15) \
                     ON CONFLICT (client_id, user_id) DO UPDATE SET rev = transactions.rev + 1 \
                     RETURNING rev"
                )
                .bind(user_id).bind(op.client_id)
                .bind(description)
                .bind(op.amount.unwrap_or(0.0))
                .bind(op.direction.unwrap_or("debit".into()))
                .bind(op.category.unwrap_or("Miscellaneous".into()))
                .bind(txn_date)
                .bind(value_date)
                .bind(op.notes)
                .bind(op.is_transfer.unwrap_or(false))
                .bind(op.deleted.unwrap_or(false))
                .bind(op.bank.unwrap_or_else(|| "Manual".into()))
                .bind(op.bank_ref)
                .bind(fingerprint).bind(&op.sms_source)
                .fetch_one(&mut *tx).await?;
                accepted.push(SyncAccepted { client_id: op.client_id, server_rev: new_rev });
            }
            Some(stored) if op.base_rev >= 0 && op.base_rev <= stored.txn.rev && is_sync_replay(&op, &stored) => {
                // A previous pull may have passed this row while the device was
                // dirty. Re-enqueue its canonical state without changing rev,
                // dates, financial fields, exclusions or manual decisions.
                sqlx::query("UPDATE transactions SET sync_rev=sync_rev WHERE client_id=$1 AND user_id=$2")
                    .bind(op.client_id).bind(user_id).execute(&mut *tx).await?;
                accepted.push(SyncAccepted { client_id: op.client_id, server_rev: stored.txn.rev });
            }
            Some(stored) if stored.txn.rev == op.base_rev
                && !stored.txn.sms_source.as_ref().is_some_and(|source| source.user_edited == Some(true)) => {
                let server_rev = stored.txn.rev;
                let new_rev = server_rev + 1;
                sqlx::query(
                    "UPDATE transactions SET rev = $1, base_rev = rev, \
                     description = COALESCE($3, description), amount = COALESCE($4, amount), \
                     direction = COALESCE($5, direction), category = COALESCE($6, category), \
                     notes = COALESCE($7, notes), is_transfer = COALESCE($8, is_transfer), \
                     deleted = COALESCE($9, deleted), txn_date = COALESCE($11, txn_date), \
                     value_date = COALESCE($12, value_date), bank = COALESCE($13, bank), \
                     bank_ref = COALESCE($14, bank_ref), sms_source = COALESCE($15, sms_source), updated_at = NOW() \
                     WHERE client_id = $2 AND user_id = $10"
                )
                .bind(new_rev).bind(op.client_id)
                .bind(&op.description).bind(op.amount)
                .bind(&op.direction).bind(&op.category)
                .bind(&op.notes).bind(op.is_transfer)
                .bind(op.deleted).bind(user_id).bind(op.txn_date).bind(op.value_date)
                .bind(&op.bank).bind(&op.bank_ref).bind(&op.sms_source)
                .execute(&mut *tx).await?;
                accepted.push(SyncAccepted { client_id: op.client_id, server_rev: new_rev });
            }
            Some(stored) => {
                conflicts.push(SyncConflictDetail {
                    client_id: op.client_id,
                    base_rev: op.base_rev,
                    server_rev: stored.txn.rev,
                    server_txn: stored.txn,
                    local_txn: serde_json::json!(op),
                });
            }
        }
    }

    finish_sync_revisions(&mut tx, user_id, &mut accepted).await?;
    for conflict in &mut conflicts {
        conflict.server_txn = sqlx::query_as(
            "SELECT id, txn_date, value_date, description, amount::float8, direction, \
             balance::float8, bank, bank_ref, category, is_transfer, notes, version, \
             rev, base_rev, deleted, updated_at, client_id, sms_source \
             FROM transactions WHERE client_id=$1 AND user_id=$2"
        ).bind(conflict.client_id).bind(user_id).fetch_one(&mut *tx).await?;
        conflict.server_rev = conflict.server_txn.rev;
    }
    tx.commit().await?;
    Ok(Json(SyncPushResult { accepted, conflicts }))
}

pub async fn sync_resolve(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(resolutions): Json<Vec<SyncPushOp>>,
) -> Result<Json<Vec<SyncAccepted>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user_id).execute(&mut *tx).await?;

    let mut accepted = vec![];
    for op in resolutions {
        if let Some(source) = &op.sms_source { source.validate()?; }
        let server_rev: Option<(i64,)> = sqlx::query_as(
            "SELECT rev FROM transactions WHERE client_id = $1 AND user_id = $2"
        ).bind(op.client_id).bind(user_id).fetch_optional(&mut *tx).await?;

        if let Some((rev,)) = server_rev {
            let new_rev = rev + 1;
            sqlx::query(
                "UPDATE transactions SET rev = $1, description = COALESCE($3, description), \
                 amount = COALESCE($4, amount), direction = COALESCE($5, direction), \
                 category = COALESCE($6, category), notes = COALESCE($7, notes), \
                 is_transfer = COALESCE($8, is_transfer), deleted = COALESCE($9, deleted), \
                 txn_date = COALESCE($11, txn_date), value_date = COALESCE($12, value_date), \
                 bank = COALESCE($13, bank), bank_ref = COALESCE($14, bank_ref), \
                 sms_source = CASE WHEN sms_source->>'user_edited'='true' THEN sms_source ELSE COALESCE($15, sms_source) END, \
                 updated_at = NOW() WHERE client_id = $2 AND user_id = $10"
            )
            .bind(new_rev).bind(op.client_id)
            .bind(&op.description).bind(op.amount)
            .bind(&op.direction).bind(&op.category)
            .bind(&op.notes).bind(op.is_transfer)
            .bind(op.deleted).bind(user_id).bind(op.txn_date).bind(op.value_date)
                .bind(&op.bank).bind(&op.bank_ref).bind(&op.sms_source)
            .execute(&mut *tx).await?;
            accepted.push(SyncAccepted { client_id: op.client_id, server_rev: new_rev });
        }
    }

    finish_sync_revisions(&mut tx, user_id, &mut accepted).await?;
    tx.commit().await?;
    Ok(Json(accepted))
}
