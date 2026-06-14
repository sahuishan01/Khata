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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version
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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version
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

// ── Analytics Explore ─────────────────────────────────────────────────────────

pub async fn get_analytics_explore(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx)
        .await?;

    let group_col = match params.group_by.as_deref() {
        Some("payee") => "description",
        Some("account") => "account_label",
        Some("week") => "to_char(value_date, 'IYYY-\"W\"IW')",
        Some("month") => "to_char(value_date, 'YYYY-MM')",
        _ => "category",
    };

    let dim_where = match params.dimension.as_deref() {
        Some("earned") => "AND direction = 'credit'".to_string(),
        Some("net") => String::new(),
        _ => "AND direction = 'debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type = 'investment')".to_string(),
    };

    let mut wheres = vec![format!("user_id = $1")];
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
        .map_err(|e| AppError::BadRequest(format!("Query error: {e}")))?;

    let total: f64 = rows.iter().map(|(_, v)| v).sum();
    let labels: Vec<String> = rows.iter().map(|(l, _)| l.clone()).collect();
    let values: Vec<f64> = rows.iter().map(|(_, v)| *v).collect();

    let comparison_values = None;

    let mut insights = vec![];
    if let Some((top_label, top_val)) = rows.first() {
        if total > 0.0 {
            let pct = top_val / total * 100.0;
            insights.push(format!(
                "{} ({:.0}%) of spending goes to {}",
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
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let mut wheres = vec!["user_id = $1".to_string()];
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

    let (total_spent, total_earned, txn_count): (f64, f64, i64) = sqlx::query_as(
        &format!("SELECT COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0)::float8, COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer),0)::float8, COUNT(*)::bigint FROM transactions WHERE {}", w)
    ).bind(user_id)
    .apply(|q| { let mut q = q; if let Some(ref v) = params.category { q = q.bind(v); } if let Some(ref v) = params.month { q = q.bind(v); } if let Some(ref v) = params.payee { q = q.bind(v); } q })
    .fetch_one(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let trend: Vec<MonthBucket> = sqlx::query_as(
        &format!("SELECT to_char(value_date, 'YYYY-MM') AS month, COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0)::float8 AS spent, COALESCE(SUM(amount) FILTER (WHERE direction='credit' AND NOT is_transfer),0)::float8 AS earned FROM transactions WHERE {} GROUP BY month ORDER BY month DESC LIMIT 12", w)
    ).bind(user_id)
    .apply(|q| { let mut q = q; if let Some(ref v) = params.category { q = q.bind(v); } if let Some(ref v) = params.month { q = q.bind(v); } if let Some(ref v) = params.payee { q = q.bind(v); } q })
    .fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let top_payees: Vec<TopMerchant> = sqlx::query_as(
        &format!("SELECT description, SUM(amount)::float8 AS total FROM transactions WHERE {} AND direction='debit' AND NOT is_transfer GROUP BY description ORDER BY total DESC LIMIT 10", w)
    ).bind(user_id)
    .apply(|q| { let mut q = q; if let Some(ref v) = params.category { q = q.bind(v); } if let Some(ref v) = params.month { q = q.bind(v); } if let Some(ref v) = params.payee { q = q.bind(v); } q })
    .fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let top_categories: Vec<CategoryBucket> = sqlx::query_as(
        &format!("SELECT category, SUM(amount)::float8 AS amount, COUNT(*)::bigint AS txn_count, 0.0 AS pct FROM transactions WHERE {} AND direction='debit' AND NOT is_transfer GROUP BY category ORDER BY amount DESC LIMIT 10", w)
    ).bind(user_id)
    .apply(|q| { let mut q = q; if let Some(ref v) = params.category { q = q.bind(v); } if let Some(ref v) = params.month { q = q.bind(v); } if let Some(ref v) = params.payee { q = q.bind(v); } q })
    .fetch_all(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    tx.commit().await?;
    Ok(Json(DetailResponse { total_spent, total_earned, txn_count, trend, top_payees, top_categories }))
}

pub async fn get_analytics_highlights(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<HighlightsResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let highest_earning_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND direction='credit' AND NOT is_transfer GROUP BY month ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let highest_spending_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND direction='debit' AND NOT is_transfer AND category NOT IN (SELECT name FROM categories WHERE user_id = $1 AND txn_type='investment') GROUP BY month ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let biggest_expense: Option<TxnRow> = sqlx::query_as(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version FROM transactions WHERE user_id = $1 AND direction='debit' AND NOT is_transfer ORDER BY amount DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let biggest_income: Option<TxnRow> = sqlx::query_as(
        "SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version FROM transactions WHERE user_id = $1 AND direction='credit' AND NOT is_transfer ORDER BY amount DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let top_payee: Option<(String, f64)> = sqlx::query_as(
        "SELECT description, SUM(amount)::float8 FROM transactions WHERE user_id = $1 AND direction='debit' AND NOT is_transfer GROUP BY description ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let most_frequent_payee: Option<(String, i64)> = sqlx::query_as(
        "SELECT description, COUNT(*)::bigint FROM transactions WHERE user_id = $1 AND direction='debit' GROUP BY description ORDER BY COUNT(*) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let top_category: Option<(String, f64, i64)> = sqlx::query_as(
        "SELECT category, SUM(amount)::float8, COUNT(*)::bigint FROM transactions WHERE user_id = $1 AND direction='debit' AND NOT is_transfer GROUP BY category ORDER BY SUM(amount) DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

    let best_savings_month: Option<(String, f64)> = sqlx::query_as(
        "SELECT to_char(value_date, 'YYYY-MM') AS month, \
         (COALESCE(SUM(amount) FILTER (WHERE direction='credit'),0) - \
          COALESCE(SUM(amount) FILTER (WHERE direction='debit' AND NOT is_transfer),0))::float8 / \
         NULLIF(SUM(amount) FILTER (WHERE direction='credit'),0) * 100.0 AS savings_rate \
         FROM transactions WHERE user_id = $1 GROUP BY month ORDER BY savings_rate DESC LIMIT 1"
    ).bind(user_id).fetch_optional(&mut *tx).await.map_err(|e| AppError::BadRequest(e.to_string()))?;

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
                  amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version
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
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

    let row = sqlx::query_as::<_, TxnRow>(
        r#"INSERT INTO transactions
           (user_id, bank, account_label, txn_date, value_date, description, raw_description,
            amount, direction, category, bank_ref, notes, version, fingerprint)
           VALUES ($1, 'Manual', 'Manual', $2, $3, $4, $4, $5, $6, $7, $8, $9, gen_random_uuid()::text)
           RETURNING id, txn_date, value_date, description,
                     amount::float8, direction, balance::float8, bank, bank_ref, category,
                     is_transfer, notes, version"#,
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

pub async fn sync_txns(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(batch): Json<Vec<SyncTxn>>,
) -> Result<Json<SyncResult>, AppError> {
    let mut tx = state.db.begin().await?;
    sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
        .execute(&mut *tx).await?;

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
                          is_transfer, notes, version
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
            sqlx::query("UPDATE transactions SET is_transfer = $1, version = version + 1 WHERE id = $2 AND user_id = $3")
                .bind(is_transfer).bind(id).bind(user_id)
                .execute(&mut *tx).await?;
        }

        success.push(item.id.clone());
    }

    tx.commit().await?;
    Ok(Json(SyncResult { success, conflicts }))
}
