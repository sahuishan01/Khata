use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
    pub owner_id: Uuid,
    pub body: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmsImportRequest {
    owner_id: Uuid,
    operation: crate::txns::models::SyncPushOp,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmsLookupRequest {
    owner_id: Uuid,
    client_id: String,
}

async fn lookup_sms(
    State(state): State<AppState>,
    CurrentUser(owner): CurrentUser,
    Json(request): Json<SmsLookupRequest>,
) -> Result<Json<Option<crate::txns::models::TxnRow>>, AppError> {
    if owner != request.owner_id {
        return Err(AppError::Forbidden);
    }
    // Legacy Android-only identities are not server UUIDs.
    let Ok(client_id) = request.client_id.parse::<Uuid>() else {
        return Ok(Json(None));
    };
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, owner).await?;
    let row = sqlx::query_as("SELECT id, txn_date, value_date, description, amount::float8, direction, balance::float8, bank, bank_ref, category, is_transfer, notes, version, rev, base_rev, deleted, updated_at, client_id, sms_source FROM transactions WHERE user_id=$1 AND client_id=$2")
        .bind(owner).bind(client_id).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(row))
}

async fn import_sms(
    State(state): State<AppState>,
    CurrentUser(owner): CurrentUser,
    Json(request): Json<SmsImportRequest>,
) -> Result<Json<crate::txns::models::TxnRow>, AppError> {
    if owner != request.owner_id {
        return Err(AppError::Forbidden);
    }
    let operation = request.operation;
    if operation.notes.as_deref().is_none_or(contains_otp) {
        return Err(AppError::BadRequest(
            "Authentication messages are not processed".into(),
        ));
    }
    if operation
        .amount
        .is_none_or(|amount| !amount.is_finite() || amount <= 0.0 || amount > 999_999_999_999.99)
        || !matches!(operation.direction.as_deref(), Some("debit" | "credit"))
        || operation
            .amount
            .is_some_and(|amount| (amount * 100.0 - (amount * 100.0).round()).abs() > 0.01)
        || operation
            .notes
            .as_ref()
            .is_none_or(|v| v.is_empty() || v.chars().count() > 16_000)
        || operation
            .description
            .as_ref()
            .is_none_or(|v| v.trim().is_empty() || v.chars().count() > 16_000)
        || operation
            .category
            .as_ref()
            .is_none_or(|v| v.trim().is_empty() || v.chars().count() > 100)
        || operation
            .bank
            .as_ref()
            .is_none_or(|v| v.trim().is_empty() || v.chars().count() > 100)
        || operation
            .bank_ref
            .as_ref()
            .is_some_and(|v| v.chars().count() > 500)
        || operation.txn_date.is_none()
        || operation.value_date.is_none()
        || operation.sms_source.is_none()
        || operation.base_rev < 0
    {
        return Err(AppError::BadRequest("Invalid SMS transaction".into()));
    }
    let client_id = operation.client_id;
    let result = crate::txns::handlers::sync_push(
        State(state.clone()),
        CurrentUser(owner),
        Json(vec![operation]),
    )
    .await?
    .0;
    if !result.conflicts.is_empty() {
        return Err(AppError::Conflict(
            "Review the saved transaction before retrying".into(),
        ));
    }
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, owner).await?;
    let id: Uuid =
        sqlx::query_scalar("SELECT id FROM transactions WHERE user_id=$1 AND client_id=$2")
            .bind(owner)
            .bind(client_id)
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;
    crate::txns::handlers::get_txn(State(state), CurrentUser(owner), Path(id)).await
}

pub fn contains_otp(body: &str) -> bool {
    static CODE: OnceLock<regex::Regex> = OnceLock::new();
    body.to_lowercase().contains("otp") || CODE.get_or_init(|| regex::Regex::new(
        r"(?i)\b(?:one[ -]?time[ -]?(?:password|passcode|code)|login[ -]?code|verification[ -]?code|authentication[ -]?code)\b"
    ).expect("constant regex")).is_match(body)
}

// Retired endpoint for older apps. Never load or contact an inference service.
async fn classify(
    CurrentUser(owner): CurrentUser,
    Json(request): Json<DecisionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if owner != request.owner_id {
        return Err(AppError::Forbidden);
    }
    Err(AppError::BadRequest(
        "SMS model decisions are disabled. Update Khata to use on-device parsing.".into(),
    ))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/decide", post(classify))
        .route("/import", post(import_sms))
        .route("/lookup", post(lookup_sms))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentication_messages_remain_blocked() {
        for body in [
            "OTP 123456 for INR 500",
            "One-time password 123456",
            "Login code 123456",
        ] {
            assert!(contains_otp(body));
        }
        assert!(!contains_otp("INR 500 sent"));
    }
}
