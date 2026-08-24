use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};
use super::models::*;

pub async fn list_subscriptions_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<SubscriptionRow>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<SubscriptionRow> = sqlx::query_as(
        "SELECT id, user_id, name, amount::double precision as amount, billing_cycle, next_due_date, category, auto_detected, active, created_at \
         FROM subscriptions WHERE user_id = $1 ORDER BY next_due_date ASC"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(rows))
}

pub async fn create_subscription_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateSubscriptionReq>,
) -> Result<Json<SubscriptionRow>, AppError> {
    let category = req.category.unwrap_or_else(|| "Subscriptions".to_string());

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: SubscriptionRow = sqlx::query_as(
        "INSERT INTO subscriptions (user_id, name, amount, billing_cycle, next_due_date, category) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, user_id, name, amount::double precision as amount, billing_cycle, next_due_date, category, auto_detected, active, created_at"
    )
    .bind(user_id)
    .bind(&req.name)
    .bind(req.amount)
    .bind(&req.billing_cycle)
    .bind(req.next_due_date)
    .bind(&category)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "subscription_created",
        Some(serde_json::json!({"name": &req.name, "amount": req.amount})),
    )
    .await;

    Ok(Json(row))
}

pub async fn delete_subscription_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let res = sqlx::query("DELETE FROM subscriptions WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({"message": "Subscription deleted"})))
}
