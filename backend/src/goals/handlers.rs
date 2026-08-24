use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};
use super::models::*;

pub async fn list_goals_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<GoalRow>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<GoalRow> = sqlx::query_as(
        "SELECT id, user_id, name, target_amount::double precision as target_amount, \
         current_amount::double precision as current_amount, target_date, color_hex, created_at \
         FROM goals WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(rows))
}

pub async fn create_goal_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateGoalReq>,
) -> Result<Json<GoalRow>, AppError> {
    let current_amount = req.current_amount.unwrap_or(0.0);
    let color_hex = req.color_hex.unwrap_or_else(|| "#6366f1".to_string());

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: GoalRow = sqlx::query_as(
        "INSERT INTO goals (user_id, name, target_amount, current_amount, target_date, color_hex) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, user_id, name, target_amount::double precision as target_amount, \
         current_amount::double precision as current_amount, target_date, color_hex, created_at"
    )
    .bind(user_id)
    .bind(&req.name)
    .bind(req.target_amount)
    .bind(current_amount)
    .bind(req.target_date)
    .bind(&color_hex)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "goal_created",
        Some(serde_json::json!({"name": &req.name, "target": req.target_amount})),
    )
    .await;

    Ok(Json(row))
}

pub async fn update_goal_progress_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGoalProgressReq>,
) -> Result<Json<GoalRow>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<GoalRow> = sqlx::query_as(
        "UPDATE goals SET current_amount = $1 \
         WHERE id = $2 AND user_id = $3 \
         RETURNING id, user_id, name, target_amount::double precision as target_amount, \
         current_amount::double precision as current_amount, target_date, color_hex, created_at"
    )
    .bind(req.current_amount)
    .bind(id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    match row {
        Some(r) => Ok(Json(r)),
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_goal_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let res = sqlx::query("DELETE FROM goals WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({"message": "Goal deleted"})))
}
