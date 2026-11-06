use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth, auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_rules(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<CategoryRule>>, AppError> {
    let rules = sqlx::query_as::<_, CategoryRule>(
        "SELECT id, user_id, pattern, category FROM category_rules WHERE user_id = $1 ORDER BY pattern",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch rules".into()))?;

    Ok(Json(rules))
}

pub async fn create_rule(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateRuleReq>,
) -> Result<Json<CategoryRule>, AppError> {
    let pattern = req.pattern.trim().to_uppercase();
    let category = req.category.trim().to_string();

    if pattern.is_empty() || category.is_empty() {
        return Err(AppError::BadRequest("Pattern and category are required".into()));
    }

    // transactions has FORCE ROW LEVEL SECURITY: without the user context the
    // UPDATE below silently matches nothing.
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rule = sqlx::query_as::<_, CategoryRule>(
        "INSERT INTO category_rules (user_id, pattern, category) VALUES ($1, $2, $3) RETURNING id, user_id, pattern, category",
    )
    .bind(user_id)
    .bind(&pattern)
    .bind(&category)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AppError::BadRequest("Failed to create rule".into()))?;

    // Apply the new rule to existing matching transactions.
    apply_one(&mut tx, user_id, &pattern, &category)
        .await
        .map_err(|_| AppError::BadRequest("Failed to apply rule".into()))?;

    tx.commit().await?;

    Ok(Json(rule))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth::verify_ownership(&state.db, rule_id, user_id, auth::OwnedTable::CategoryRules).await?;

    sqlx::query("DELETE FROM category_rules WHERE id = $1")
        .bind(rule_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete rule".into()))?;

    Ok(Json(serde_json::json!({ "message": "Rule deleted" })))
}

pub async fn apply_rules(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let rules = sqlx::query_as::<_, CategoryRule>(
        "SELECT id, user_id, pattern, category FROM category_rules WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch rules".into()))?;

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let mut total_updated = 0u64;
    for rule in &rules {
        total_updated += apply_one(&mut tx, user_id, &rule.pattern, &rule.category)
            .await
            .map_err(|_| AppError::BadRequest("Failed to apply rules".into()))?;
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "message": format!("Applied {} rules, updated {} transactions", rules.len(), total_updated),
        "rules_count": rules.len(),
        "updated": total_updated
    })))
}

/// Recategorize the user's transactions whose description contains `pattern`.
/// Transfers and investment-category rows are left alone. The caller MUST have
/// set the RLS user context on `conn`.
async fn apply_one(
    conn: &mut sqlx::PgConnection,
    user_id: Uuid,
    pattern: &str,
    category: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE transactions SET category = $1 \
         WHERE user_id = $2 AND UPPER(description) LIKE $3 AND NOT is_transfer \
           AND category NOT IN (SELECT name FROM categories WHERE user_id = $2 AND txn_type = 'investment')",
    )
    .bind(category)
    .bind(user_id)
    .bind(format!("%{}%", pattern.to_uppercase()))
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{app_state, seed_txn, seed_user, txn_field};
    use chrono::NaiveDate;
    use sqlx::PgPool;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()
    }

    /// transactions is FORCE RLS: a rule must actually recategorize the
    /// user's existing rows, not silently match nothing.
    #[sqlx::test(migrations = "./migrations")]
    async fn create_rule_recategorizes_existing_rows(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let id = seed_txn(&pool, uid, "HDFC", "UPI/ZEPTO/order", "debit", 250.0, day(), "Miscellaneous").await;

        create_rule(
            State(app_state(&pool)),
            CurrentUser(uid),
            Json(CreateRuleReq { pattern: "zepto".into(), category: "Groceries".into() }),
        )
        .await
        .expect("create_rule");

        let cat: String = txn_field(&pool, uid, id, "category").await;
        assert_eq!(cat, "Groceries");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn apply_rules_updates_rows_and_skips_other_users(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let other = seed_user(&pool).await;
        let mine = seed_txn(&pool, uid, "HDFC", "NETFLIX.COM", "debit", 649.0, day(), "Miscellaneous").await;
        let theirs = seed_txn(&pool, other, "HDFC", "NETFLIX.COM", "debit", 649.0, day(), "Miscellaneous").await;
        sqlx::query("INSERT INTO category_rules (user_id, pattern, category) VALUES ($1,'NETFLIX','Entertainment')")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();

        let Json(res) = apply_rules(State(app_state(&pool)), CurrentUser(uid)).await.expect("apply_rules");
        assert_eq!(res["updated"], 1);

        let mine_cat: String = txn_field(&pool, uid, mine, "category").await;
        let theirs_cat: String = txn_field(&pool, other, theirs, "category").await;
        assert_eq!(mine_cat, "Entertainment");
        assert_eq!(theirs_cat, "Miscellaneous");
    }
}
