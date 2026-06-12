use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_categories(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<Category>>, AppError> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, user_id, name, txn_type, color, description FROM categories WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch categories".into()))?;
    Ok(Json(rows))
}

pub async fn create_category(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateCategoryReq>,
) -> Result<Json<Category>, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }
    if req.txn_type != "income" && req.txn_type != "expense" {
        return Err(AppError::BadRequest("txn_type must be income or expense".into()));
    }

    let row = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (user_id, name, txn_type, color, description) VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, name, txn_type, color, description",
    )
    .bind(user_id)
    .bind(&name)
    .bind(&req.txn_type)
    .bind(req.color.as_deref().unwrap_or("#6C5CE7"))
    .bind(req.description.as_deref().unwrap_or(""))
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.code().as_deref() == Some("23505") {
                return AppError::Conflict("A category with this name already exists".into());
            }
        }
        AppError::BadRequest("Failed to create category".into())
    })?;
    Ok(Json(row))
}

pub async fn delete_category(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(cat_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("DELETE FROM categories WHERE id = $1 AND user_id = $2")
        .bind(cat_id).bind(user_id)
        .execute(&state.db).await
        .map_err(|_| AppError::BadRequest("Failed".into()))?;
    if r.rows_affected() == 0 { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "message": "Category deleted" })))
}
