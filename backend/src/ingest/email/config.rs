//! Per-user Gmail connection config: `GET` / `PUT` / `DELETE
//! /api/ingest/email/config`.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};

use crate::ingest::crypto;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEmailConfigResponse {
    pub email_address: String,
    pub imap_server: String,
    pub imap_folder: String,
    pub sync_enabled: bool,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub has_app_password: bool,
    pub has_pdf_password: bool,
    #[serde(default)]
    pub sender_allowlist: Vec<String>,
    #[serde(default)]
    pub subject_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveEmailConfigReq {
    pub email_address: String,
    /// Omitted or blank keeps the stored app password.
    pub app_password: Option<String>,
    pub pdf_password: Option<String>,
    pub imap_server: Option<String>,
    /// IMAP folder to scan. Defaults to `[Gmail]/All Mail` (statements are
    /// usually auto-archived out of the inbox).
    pub imap_folder: Option<String>,
    pub sync_enabled: Option<bool>,
    /// Optional IMAP `FROM` filter — any match. Replaces the stored list when present.
    pub sender_allowlist: Option<Vec<String>>,
    /// Optional IMAP `SUBJECT` filter — any match. Replaces the stored list when present.
    pub subject_patterns: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct SaveEmailConfigResponse {
    #[serde(flatten)]
    pub config: UserEmailConfigResponse,
    pub full_rescan_queued: bool,
}

pub async fn get_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Option<UserEmailConfigResponse>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(
        String, String, String, bool, Option<chrono::DateTime<chrono::Utc>>, Option<String>,
        bool, bool, Option<Vec<String>>, Option<Vec<String>>,
    )> = sqlx::query_as(
        "SELECT email_address, imap_server, imap_folder, sync_enabled, last_synced_at, last_error, \
                (encrypted_app_password IS NOT NULL AND encrypted_app_password <> ''), \
                (encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> ''), \
                sender_allowlist, subject_patterns \
         FROM user_email_configs WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(row.map(|(
        email, imap, folder, enabled, last_synced, last_err, has_app, has_pdf, senders, subjects,
    )| UserEmailConfigResponse {
        email_address: email,
        imap_server: imap,
        imap_folder: folder,
        sync_enabled: enabled,
        last_synced_at: last_synced,
        last_error: last_err,
        has_app_password: has_app,
        has_pdf_password: has_pdf,
        sender_allowlist: senders.unwrap_or_default(),
        subject_patterns: subjects.unwrap_or_default(),
    })))
}

pub async fn save_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<SaveEmailConfigReq>,
) -> Result<Json<SaveEmailConfigResponse>, AppError> {
    let secret = &state.config.jwt_secret;
    let uid = user_id.to_string();

    let new_app_pass = match req.app_password {
        Some(ref p) if !p.trim().is_empty() => Some(crypto::encrypt_credential(p.trim(), secret, &uid)?),
        _ => None,
    };
    let new_pdf_pass = match req.pdf_password {
        Some(ref p) if !p.trim().is_empty() => Some(crypto::encrypt_credential(p.trim(), secret, &uid)?),
        _ => None,
    };
    let imap = req.imap_server.clone().unwrap_or_else(|| "imap.gmail.com:993".to_string());
    let folder = req
        .imap_folder
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .unwrap_or("[Gmail]/All Mail")
        .to_string();
    let enabled = req.sync_enabled.unwrap_or(true);

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let existing: Option<(bool,)> = sqlx::query_as(
        "SELECT (encrypted_app_password IS NOT NULL AND encrypted_app_password <> '') \
         FROM user_email_configs WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let had_app_password = existing.map(|(a,)| a).unwrap_or(false);

    if new_app_pass.is_none() && !had_app_password {
        tx.rollback().await?;
        return Err(AppError::BadRequest("A Google App Password is required to connect Gmail.".into()));
    }

    // Rotating the app password ⇒ reset the watermark for a full rescan.
    let full_rescan_queued = new_app_pass.is_some();

    sqlx::query(
        "INSERT INTO user_email_configs \
         (user_id, email_address, encrypted_app_password, encrypted_pdf_password, imap_server, \
          imap_folder, sync_enabled, sender_allowlist, subject_patterns) \
         VALUES ($1,$2,COALESCE($3,''),$4,$5,$6,$7,$8,$9) \
         ON CONFLICT (user_id) DO UPDATE SET \
            email_address = EXCLUDED.email_address, \
            encrypted_app_password = COALESCE($3, user_email_configs.encrypted_app_password), \
            encrypted_pdf_password = COALESCE($4, user_email_configs.encrypted_pdf_password), \
            imap_server = EXCLUDED.imap_server, \
            imap_folder = EXCLUDED.imap_folder, \
            sync_enabled = EXCLUDED.sync_enabled, \
            sender_allowlist = COALESCE($8, user_email_configs.sender_allowlist), \
            subject_patterns = COALESCE($9, user_email_configs.subject_patterns), \
            last_error = NULL, \
            last_synced_at = CASE WHEN $10 THEN NULL ELSE user_email_configs.last_synced_at END, \
            last_uid = CASE WHEN $10 THEN NULL ELSE user_email_configs.last_uid END",
    )
    .bind(user_id)
    .bind(&req.email_address)
    .bind(&new_app_pass)
    .bind(&new_pdf_pass)
    .bind(&imap)
    .bind(&folder)
    .bind(enabled)
    .bind(req.sender_allowlist.as_deref())
    .bind(req.subject_patterns.as_deref())
    .bind(full_rescan_queued)
    .execute(&mut *tx)
    .await?;

    let (has_pdf, senders, subjects): (bool, Option<Vec<String>>, Option<Vec<String>>) = sqlx::query_as(
        "SELECT (encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> ''), \
                sender_allowlist, subject_patterns \
         FROM user_email_configs WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "email_config_saved",
        Some(serde_json::json!({"email": &req.email_address, "imap": &imap, "key_rotated": full_rescan_queued})),
    )
    .await;

    Ok(Json(SaveEmailConfigResponse {
        config: UserEmailConfigResponse {
            email_address: req.email_address,
            imap_server: imap,
            imap_folder: folder,
            sync_enabled: enabled,
            last_synced_at: None,
            last_error: None,
            has_app_password: true,
            has_pdf_password: has_pdf,
            sender_allowlist: senders.unwrap_or_default(),
            subject_patterns: subjects.unwrap_or_default(),
        },
        full_rescan_queued,
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
