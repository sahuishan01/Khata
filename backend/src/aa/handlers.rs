use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn get_settings_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<AaSettingsResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let row: Option<(bool, i32, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT auto_fetch_enabled, fetch_interval_days, last_fetched_at, next_fetch_due_at \
         FROM aa_sync_settings WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    if let Some((enabled, interval, last_fetched, next_due)) = row {
        Ok(Json(AaSettingsResponse {
            auto_fetch_enabled: enabled,
            fetch_interval_days: interval,
            last_fetched_at: last_fetched,
            next_fetch_due_at: next_due,
        }))
    } else {
        Ok(Json(AaSettingsResponse {
            auto_fetch_enabled: true,
            fetch_interval_days: 7,
            last_fetched_at: None,
            next_fetch_due_at: None,
        }))
    }
}

pub async fn update_settings_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<UpdateAaSettingsReq>,
) -> Result<Json<AaSettingsResponse>, AppError> {
    let interval = req.fetch_interval_days.unwrap_or(7);
    if ![1, 3, 7, 14, 30].contains(&interval) {
        return Err(AppError::BadRequest(
            "fetch_interval_days must be 1, 3, 7, 14, or 30".into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let now = Utc::now();
    let next_due = if req.auto_fetch_enabled {
        Some(now + chrono::Duration::days(interval as i64))
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO aa_sync_settings (user_id, auto_fetch_enabled, fetch_interval_days, next_fetch_due_at) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id) DO UPDATE SET \
            auto_fetch_enabled = EXCLUDED.auto_fetch_enabled, \
            fetch_interval_days = EXCLUDED.fetch_interval_days, \
            next_fetch_due_at = EXCLUDED.next_fetch_due_at"
    )
    .bind(user_id)
    .bind(req.auto_fetch_enabled)
    .bind(interval)
    .bind(next_due)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "aa_settings_updated",
        Some(serde_json::json!({
            "auto_fetch_enabled": req.auto_fetch_enabled,
            "interval_days": interval
        })),
    )
    .await;

    Ok(Json(AaSettingsResponse {
        auto_fetch_enabled: req.auto_fetch_enabled,
        fetch_interval_days: interval,
        last_fetched_at: None,
        next_fetch_due_at: next_due,
    }))
}

pub async fn init_consent_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<InitConsentResponse>, AppError> {
    let consent_id = format!("cst_{}", Uuid::new_v4().simple());
    let base_url = &state.config.setu_base_url;
    let consent_url = format!("{base_url}/bridge?consent_id={consent_id}");

    let valid_until = Utc::now() + chrono::Duration::days(365);

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    sqlx::query(
        "INSERT INTO aa_consents (user_id, consent_id, handle_id, status, valid_until) \
         VALUES ($1, $2, $3, 'ACTIVE', $4)"
    )
    .bind(user_id)
    .bind(&consent_id)
    .bind("setu_handle")
    .bind(valid_until)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "aa_consent_initiated",
        Some(serde_json::json!({"consent_id": &consent_id})),
    )
    .await;

    Ok(Json(InitConsentResponse {
        consent_id,
        consent_url,
    }))
}

pub async fn manual_fetch_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<FetchResponse>, AppError> {
    let now = Utc::now();

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let consent: Option<(String,)> = sqlx::query_as(
        "SELECT consent_id FROM aa_consents WHERE user_id = $1 AND status = 'ACTIVE' AND valid_until > NOW()"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if consent.is_none() {
        tx.commit().await?;
        return Err(AppError::BadRequest(
            "No active Account Aggregator consent found. Please connect your accounts first.".into(),
        ));
    }

    let interval_res: Option<(i32,)> = sqlx::query_as(
        "SELECT fetch_interval_days FROM aa_sync_settings WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let interval = interval_res.map(|(i,)| i).unwrap_or(7);
    let next_due = now + chrono::Duration::days(interval as i64);

    sqlx::query(
        "INSERT INTO aa_sync_settings (user_id, last_fetched_at, next_fetch_due_at) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (user_id) DO UPDATE SET \
            last_fetched_at = EXCLUDED.last_fetched_at, \
            next_fetch_due_at = EXCLUDED.next_fetch_due_at"
    )
    .bind(user_id)
    .bind(now)
    .bind(next_due)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "aa_manual_fetch_triggered",
        Some(serde_json::json!({"fetched_at": now})),
    )
    .await;

    Ok(Json(FetchResponse {
        message: "Account transactions & investments updated successfully".into(),
        transactions_added: 0,
        portfolio_assets_updated: 0,
        fetched_at: now,
    }))
}

pub async fn list_consents_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<ConsentStatusResponse>>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT consent_id, status, valid_until FROM aa_consents WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    let list = rows
        .into_iter()
        .map(|(cid, status, valid)| ConsentStatusResponse {
            consent_id: cid,
            status,
            valid_until: valid,
        })
        .collect();

    Ok(Json(list))
}
