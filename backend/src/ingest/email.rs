use axum::{
    extract::State,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    audit, auth::middleware::CurrentUser, error::AppError, AppState,
};

use super::crypto;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEmailConfigResponse {
    pub email_address: String,
    pub imap_server: String,
    pub sync_enabled: bool,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveEmailConfigReq {
    pub email_address: String,
    pub app_password: String,
    pub pdf_password: Option<String>,
    pub imap_server: Option<String>,
    pub sync_enabled: Option<bool>,
}

pub async fn get_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Option<UserEmailConfigResponse>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(String, String, bool, Option<chrono::DateTime<chrono::Utc>>, Option<String>)> = sqlx::query_as(
        "SELECT email_address, imap_server, sync_enabled, last_synced_at, last_error \
         FROM user_email_configs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    if let Some((email, imap, enabled, last_synced, last_err)) = row {
        Ok(Json(Some(UserEmailConfigResponse {
            email_address: email,
            imap_server: imap,
            sync_enabled: enabled,
            last_synced_at: last_synced,
            last_error: last_err,
        })))
    } else {
        Ok(Json(None))
    }
}

pub async fn save_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<SaveEmailConfigReq>,
) -> Result<Json<UserEmailConfigResponse>, AppError> {
    let secret = &state.config.jwt_secret;
    let user_id_str = user_id.to_string();

    let enc_app_pass = crypto::encrypt_credential(&req.app_password, secret, &user_id_str)?;
    let enc_pdf_pass = match req.pdf_password {
        Some(ref p) if !p.trim().is_empty() => Some(crypto::encrypt_credential(p, secret, &user_id_str)?),
        _ => None,
    };

    let imap = req.imap_server.unwrap_or_else(|| "imap.gmail.com:993".to_string());
    let enabled = req.sync_enabled.unwrap_or(true);

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    sqlx::query(
        "INSERT INTO user_email_configs \
         (user_id, email_address, encrypted_app_password, encrypted_pdf_password, imap_server, sync_enabled) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET \
            email_address = EXCLUDED.email_address, \
            encrypted_app_password = EXCLUDED.encrypted_app_password, \
            encrypted_pdf_password = EXCLUDED.encrypted_pdf_password, \
            imap_server = EXCLUDED.imap_server, \
            sync_enabled = EXCLUDED.sync_enabled, \
            last_error = NULL"
    )
    .bind(user_id)
    .bind(&req.email_address)
    .bind(&enc_app_pass)
    .bind(&enc_pdf_pass)
    .bind(&imap)
    .bind(enabled)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "email_config_saved",
        Some(serde_json::json!({"email": &req.email_address, "imap": &imap})),
    )
    .await;

    Ok(Json(UserEmailConfigResponse {
        email_address: req.email_address,
        imap_server: imap,
        sync_enabled: enabled,
        last_synced_at: None,
        last_error: None,
    }))
}

pub async fn delete_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let res = sqlx::query("DELETE FROM user_email_configs WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({"message": "Email configuration disconnected"})))
}

pub async fn trigger_email_sync_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let now = Utc::now();

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT email_address, encrypted_app_password, imap_server, encrypted_pdf_password \
         FROM user_email_configs WHERE user_id = $1 AND sync_enabled = true"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if row.is_none() {
        tx.commit().await?;
        return Err(AppError::BadRequest("No active email ingestion configuration found. Please connect your Gmail first.".into()));
    }

    sqlx::query(
        "UPDATE user_email_configs SET last_synced_at = $1, last_error = NULL WHERE user_id = $2"
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "email_sync_triggered",
        Some(serde_json::json!({"synced_at": now})),
    )
    .await;

    Ok(Json(serde_json::json!({
        "message": "Email statement sync triggered successfully",
        "synced_at": now
    })))
}
