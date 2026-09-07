//! `POST /api/ingest/email/sync`, `GET /api/ingest/email/runs`,
//! `GET /api/ingest/email/runs/latest`.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::run::{sync_user, Trigger};

#[derive(Serialize, sqlx::FromRow)]
pub struct EmailSyncRun {
    pub id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub trigger: String,
    pub full_scan: bool,
    pub messages_scanned: i32,
    pub attachments_seen: i32,
    pub attachments_parsed: i32,
    pub txns_imported: i32,
    pub txns_skipped: i32,
    pub errors: serde_json::Value,
    pub error: Option<String>,
}

const RUN_COLUMNS: &str = "id, started_at, finished_at, status, trigger, full_scan, \
    messages_scanned, attachments_seen, attachments_parsed, txns_imported, txns_skipped, \
    errors, error";

pub async fn trigger_email_sync_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    match sync_user(state.clone(), user_id, Trigger::Manual).await {
        Ok(run_id) => Ok(Json(serde_json::json!({
            "message": "Sync started",
            "run_id": run_id,
        }))),
        Err(e) => {
            let msg = format!("{e:#}");
            if msg.contains("already running") {
                Err(AppError::Conflict(msg))
            } else {
                Err(AppError::BadRequest(msg))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct RunsQuery {
    limit: Option<i64>,
}

pub async fn list_runs_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(q): Query<RunsQuery>,
) -> Result<Json<Vec<EmailSyncRun>>, AppError> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50);
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let runs: Vec<EmailSyncRun> = sqlx::query_as(&format!(
        "SELECT {RUN_COLUMNS} FROM email_sync_runs WHERE user_id = $1 \
         ORDER BY started_at DESC LIMIT $2"
    ))
    .bind(user_id)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(runs))
}

pub async fn latest_run_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Option<EmailSyncRun>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;
    let run: Option<EmailSyncRun> = sqlx::query_as(&format!(
        "SELECT {RUN_COLUMNS} FROM email_sync_runs WHERE user_id = $1 \
         ORDER BY started_at DESC LIMIT 1"
    ))
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(run))
}
