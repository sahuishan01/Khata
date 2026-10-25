use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{sync::OnceLock, time::Duration};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
    pub owner_id: Uuid,
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DecisionResponse {
    pub status: String,
    pub is_transaction: Option<bool>,
    pub is_debit: Option<bool>,
    pub transaction_probability: Option<f64>,
    pub debit_probability: Option<f64>,
    pub source_hash: String,
    pub model_version: String,
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

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(25))
            .build()
            .expect("local classifier HTTP client")
    })
}

fn local_endpoint(value: &str) -> Result<reqwest::Url, AppError> {
    let url = reqwest::Url::parse(value).map_err(|_| AppError::Internal)?;
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || !url.host_str().is_some_and(|host| {
            host.parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
        })
    {
        return Err(AppError::Internal);
    }
    Ok(url)
}

pub async fn classify_at(
    owner: Uuid,
    request: DecisionRequest,
    endpoint: &str,
) -> Result<DecisionResponse, AppError> {
    // Do not forward or reflect OTP contents, even if an older/custom client bypasses its gate.
    if contains_otp(&request.body) {
        return Err(AppError::BadRequest(
            "Authentication messages are not processed".into(),
        ));
    }
    if owner != request.owner_id {
        return Err(AppError::Forbidden);
    }
    if request.body.is_empty() || request.body.chars().count() > 16_000 {
        return Err(AppError::BadRequest(
            "SMS must contain 1-16000 characters".into(),
        ));
    }
    let mut response = client()
        .post(local_endpoint(endpoint)?)
        .json(&serde_json::json!({"body":request.body}))
        .send()
        .await
        .map_err(|_| AppError::Unavailable)?;
    if !response.status().is_success() {
        return Err(AppError::Unavailable);
    }
    if response.content_length().is_some_and(|len| len > 32_768) {
        return Err(AppError::Unavailable);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| AppError::Unavailable)? {
        if bytes.len() + chunk.len() > 32_768 {
            return Err(AppError::Unavailable);
        }
        bytes.extend_from_slice(&chunk);
    }
    let result: DecisionResponse =
        serde_json::from_slice(&bytes).map_err(|_| AppError::Unavailable)?;
    let valid_probability =
        |value: Option<f64>| value.is_none_or(|p| p.is_finite() && (0.0..=1.0).contains(&p));
    let consistent = match result.status.as_str() {
        "decision" => {
            result.is_transaction == Some(true)
                && result.is_debit.is_some()
                && result.transaction_probability.is_some_and(|p| p >= 0.75)
                && result
                    .debit_probability
                    .is_some_and(|p| p.max(1.0 - p) >= 0.55 && result.is_debit == Some(p > 0.5))
        }
        "ignored" => {
            result.is_transaction == Some(false)
                && result.is_debit.is_none()
                && result.debit_probability.is_none()
                && result.transaction_probability.is_some_and(|p| p <= 0.2)
        }
        "review" => result.is_debit.is_none(),
        _ => false,
    };
    if !consistent
        || result.source_hash != format!("{:x}", Sha256::digest(request.body.as_bytes()))
        || !result.model_version.starts_with("laya:")
        || result.model_version.len() > 128
        || !valid_probability(result.transaction_probability)
        || !valid_probability(result.debit_probability)
    {
        return Err(AppError::Unavailable);
    }
    Ok(result)
}

async fn classify(
    CurrentUser(owner): CurrentUser,
    Json(request): Json<DecisionRequest>,
) -> Result<Json<DecisionResponse>, AppError> {
    static SLOTS: OnceLock<tokio::sync::Semaphore> = OnceLock::new();
    let _permit = SLOTS
        .get_or_init(|| tokio::sync::Semaphore::new(2))
        .try_acquire()
        .map_err(|_| AppError::TooManyRequests)?;
    let endpoint =
        std::env::var("KHATA_LAYA_URL").unwrap_or_else(|_| "http://127.0.0.1:8091/decide".into());
    classify_at(owner, request, &endpoint).await.map(Json)
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

    #[tokio::test]
    async fn otp_is_rejected_before_any_endpoint_can_be_contacted() {
        for body in [
            "OTP 123456 for INR 500",
            "One-time password 123456",
            "Login code 123456",
        ] {
            let owner = Uuid::new_v4();
            let result = classify_at(
                owner,
                DecisionRequest {
                    owner_id: owner,
                    body: body.into(),
                },
                "invalid",
            )
            .await;
            assert!(matches!(result, Err(AppError::BadRequest(_))));
        }
    }

    #[tokio::test]
    async fn a_changed_login_cannot_process_another_owners_queue() {
        let result = classify_at(
            Uuid::new_v4(),
            DecisionRequest {
                owner_id: Uuid::new_v4(),
                body: "INR 500 debited".into(),
            },
            "invalid",
        )
        .await;
        assert!(matches!(result, Err(AppError::Forbidden)));
    }

    #[test]
    fn model_calls_cannot_leave_the_host() {
        for endpoint in [
            "https://127.0.0.1/",
            "http://example.com/",
            "http://0.0.0.0/",
            "http://user:password@127.0.0.1/",
        ] {
            assert!(local_endpoint(endpoint).is_err());
        }
        assert!(local_endpoint("http://127.0.0.1:8091/decide").is_ok());
    }
}
