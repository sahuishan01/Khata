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
    /// True when an app password is already stored for this user.
    pub has_app_password: bool,
    /// True when a statement (PDF) password is already stored.
    pub has_pdf_password: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveEmailConfigReq {
    pub email_address: String,
    /// Optional: when omitted or blank, the existing stored app password is kept.
    pub app_password: Option<String>,
    pub pdf_password: Option<String>,
    pub imap_server: Option<String>,
    pub sync_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SaveEmailConfigResponse {
    #[serde(flatten)]
    pub config: UserEmailConfigResponse,
    /// True when a new/updated key was stored and a full historical rescan was queued.
    pub full_rescan_queued: bool,
}

pub async fn get_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Option<UserEmailConfigResponse>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(String, String, bool, Option<chrono::DateTime<chrono::Utc>>, Option<String>, bool, bool)> = sqlx::query_as(
        "SELECT email_address, imap_server, sync_enabled, last_synced_at, last_error, \
                (encrypted_app_password IS NOT NULL AND encrypted_app_password <> '') AS has_app_password, \
                (encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> '') AS has_pdf_password \
         FROM user_email_configs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    if let Some((email, imap, enabled, last_synced, last_err, has_app_pw, has_pdf_pw)) = row {
        Ok(Json(Some(UserEmailConfigResponse {
            email_address: email,
            imap_server: imap,
            sync_enabled: enabled,
            last_synced_at: last_synced,
            last_error: last_err,
            has_app_password: has_app_pw,
            has_pdf_password: has_pdf_pw,
        })))
    } else {
        Ok(Json(None))
    }
}

pub async fn save_email_config_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<SaveEmailConfigReq>,
) -> Result<Json<SaveEmailConfigResponse>, AppError> {
    let secret = &state.config.jwt_secret;
    let user_id_str = user_id.to_string();

    let new_app_pass = match req.app_password {
        Some(ref p) if !p.trim().is_empty() => Some(crypto::encrypt_credential(p.trim(), secret, &user_id_str)?),
        _ => None,
    };
    let new_pdf_pass = match req.pdf_password {
        Some(ref p) if !p.trim().is_empty() => Some(crypto::encrypt_credential(p.trim(), secret, &user_id_str)?),
        _ => None,
    };

    let imap = req.imap_server.unwrap_or_else(|| "imap.gmail.com:993".to_string());
    let enabled = req.sync_enabled.unwrap_or(true);

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let existing: Option<(bool, bool)> = sqlx::query_as(
        "SELECT (encrypted_app_password IS NOT NULL AND encrypted_app_password <> ''), \
                (encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> '') \
         FROM user_email_configs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let had_app_password = existing.map(|(a, _)| a).unwrap_or(false);

    if new_app_pass.is_none() && !had_app_password {
        tx.rollback().await?;
        return Err(AppError::BadRequest("A Google App Password is required to connect Gmail.".into()));
    }

    // A brand-new or rotated key => reset last_synced_at so the next sync
    // rescans the full mailbox history rather than only new mail.
    let full_rescan_queued = new_app_pass.is_some();

    // COALESCE keeps the stored credential when the client omits it.
    sqlx::query(
        "INSERT INTO user_email_configs \
         (user_id, email_address, encrypted_app_password, encrypted_pdf_password, imap_server, sync_enabled) \
         VALUES ($1, $2, COALESCE($3, ''), $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET \
            email_address = EXCLUDED.email_address, \
            encrypted_app_password = COALESCE($3, user_email_configs.encrypted_app_password), \
            encrypted_pdf_password = COALESCE($4, user_email_configs.encrypted_pdf_password), \
            imap_server = EXCLUDED.imap_server, \
            sync_enabled = EXCLUDED.sync_enabled, \
            last_error = NULL, \
            last_synced_at = CASE WHEN $7 THEN NULL ELSE user_email_configs.last_synced_at END"
    )
    .bind(user_id)
    .bind(&req.email_address)
    .bind(&new_app_pass)
    .bind(&new_pdf_pass)
    .bind(&imap)
    .bind(enabled)
    .bind(full_rescan_queued)
    .execute(&mut *tx)
    .await?;

    let has_pdf_password: bool = sqlx::query_scalar(
        "SELECT (encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> '') \
         FROM user_email_configs WHERE user_id = $1"
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
            sync_enabled: enabled,
            last_synced_at: None,
            last_error: None,
            has_app_password: true,
            has_pdf_password,
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

pub async fn trigger_email_sync_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let now = Utc::now();

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(String, String, String, Option<String>, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT email_address, encrypted_app_password, imap_server, encrypted_pdf_password, last_synced_at \
         FROM user_email_configs WHERE user_id = $1 AND sync_enabled = true"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((_, _, _, _, last_synced_at)) = row else {
        tx.commit().await?;
        return Err(AppError::BadRequest("No active email ingestion configuration found. Please connect your Gmail first.".into()));
    };

    // No prior successful sync => scan the full mailbox history.
    let full_scan = last_synced_at.is_none();

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
        Some(serde_json::json!({"synced_at": now, "full_scan": full_scan})),
    )
    .await;

    Ok(Json(serde_json::json!({
        "message": if full_scan {
            "Full mailbox scan triggered — importing all historical statement transactions"
        } else {
            "Email statement sync triggered successfully"
        },
        "synced_at": now,
        "full_scan": full_scan
    })))
}
