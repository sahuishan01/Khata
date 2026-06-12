use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

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

    let rule = sqlx::query_as::<_, CategoryRule>(
        "INSERT INTO category_rules (user_id, pattern, category) VALUES ($1, $2, $3) RETURNING id, user_id, pattern, category",
    )
    .bind(user_id)
    .bind(&pattern)
    .bind(&category)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to create rule".into()))?;

    // Apply rule to all matching transactions
    let _ = sqlx::query(
        "UPDATE transactions SET category = $1 WHERE user_id = $2 AND UPPER(description) LIKE $3 AND NOT is_transfer"
    )
    .bind(&category)
    .bind(user_id)
    .bind(format!("%{}%", pattern))
    .execute(&state.db)
    .await;

    Ok(Json(rule))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM category_rules WHERE id = $1 AND user_id = $2")
        .bind(rule_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete rule".into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

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

    let mut total_updated = 0u64;
    for rule in &rules {
        let pattern = format!("%{}%", rule.pattern);
        let result = sqlx::query(
            "UPDATE transactions SET category = $1 WHERE user_id = $2 AND UPPER(description) LIKE $3 AND NOT is_transfer AND NOT is_investment"
        )
        .bind(&rule.category)
        .bind(user_id)
        .bind(&pattern)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to apply rules".into()))?;
        total_updated += result.rows_affected();
    }

    Ok(Json(serde_json::json!({
        "message": format!("Applied {} rules, updated {} transactions", rules.len(), total_updated),
        "rules_count": rules.len(),
        "updated": total_updated
    })))
}
