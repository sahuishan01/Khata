use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_accounts(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<UserAccount>>, AppError> {
    let rows = sqlx::query_as::<_, UserAccount>(
        "SELECT id, user_id, label, identifier FROM user_accounts WHERE user_id = $1 ORDER BY label",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch accounts".into()))?;

    Ok(Json(rows))
}

pub async fn create_account(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateAccountReq>,
) -> Result<Json<UserAccount>, AppError> {
    if req.label.trim().is_empty() || req.identifier.trim().is_empty() {
        return Err(AppError::BadRequest("Label and identifier are required".into()));
    }

    let account = sqlx::query_as::<_, UserAccount>(
        "INSERT INTO user_accounts (user_id, label, identifier) VALUES ($1, $2, $3) RETURNING id, user_id, label, identifier",
    )
    .bind(user_id)
    .bind(req.label.trim())
    .bind(req.identifier.trim())
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.code().as_deref() == Some("23505") {
                return AppError::Conflict("This identifier already exists".into());
            }
        }
        AppError::BadRequest("Failed to create account".into())
    })?;

    Ok(Json(account))
}

pub async fn delete_account(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM user_accounts WHERE id = $1 AND user_id = $2")
        .bind(account_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete account".into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "message": "Account deleted" })))
}
