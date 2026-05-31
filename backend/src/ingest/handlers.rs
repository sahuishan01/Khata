use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::{
    detect::{detect_bank, detect_file_kind},
    models::UploadResponse,
    normalize::normalize,
    parse::parse_file,
    profiles::registry,
    store::store_transactions,
};

// ── Upload ────────────────────────────────────────────────────────────────────

pub async fn upload_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let profiles = registry();

    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or("upload").to_string();
        let bytes = field.bytes().await?;
        let sha = hex::encode(Sha256::digest(&bytes));

        // All DB ops in one transaction so SET LOCAL covers every query
        let mut tx = state.db.begin().await?;
        sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
            .execute(&mut *tx)
            .await?;

        let exists: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM statements WHERE user_id = $1 AND file_sha256 = $2",
        )
        .bind(user_id)
        .bind(&sha)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_some() {
            tx.rollback().await?;
            return Err(AppError::Conflict(format!("{filename} has already been imported")));
        }

        let kind = detect_file_kind(&filename);

        // First pass: generic profile → get full file hint text for bank detection
        let (_, _, file_hint) = parse_file(&bytes, kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let profile = detect_bank(&profiles, &file_hint);

        // Second pass: correct profile
        let (raw_rows, _, _) = parse_file(&bytes, kind, profile)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let rows_parsed = raw_rows.len();
        if rows_parsed == 0 {
            tx.rollback().await?;
            return Err(AppError::BadRequest("No transaction rows found in file".into()));
        }

        let (stmt_id,): (uuid::Uuid,) = sqlx::query_as(
            "INSERT INTO statements (user_id, bank, file_name, file_sha256, row_count) \
             VALUES ($1,$2,$3,$4,$5) RETURNING id",
        )
        .bind(user_id)
        .bind(profile.name)
        .bind(&filename)
        .bind(&sha)
        .bind(rows_parsed as i32)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        let txns = normalize(raw_rows, user_id, stmt_id, profile, "default");
        let normalized = txns.len();

        if normalized == 0 {
            return Ok(Json(UploadResponse {
                bank_detected: profile.name.to_string(),
                rows_parsed,
                normalized: 0,
                inserted: 0,
                skipped_duplicates: 0,
            }));
        }

        let (inserted, skipped_duplicates) =
            store_transactions(&state.db, user_id, &txns).await?;

        return Ok(Json(UploadResponse {
            bank_detected: profile.name.to_string(),
            rows_parsed,
            normalized,
            inserted,
            skipped_duplicates,
        }));
    }

    Err(AppError::BadRequest("No file in upload".into()))
}

// ── Debug headers ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DebugHeadersResponse {
    pub filename: String,
    pub bank_detected: String,
    pub header_row: Vec<String>,
    pub first_data_rows: Vec<Vec<String>>,
    pub col_map: serde_json::Value,
    pub sample_normalized: usize,
}

pub async fn debug_headers_handler(
    _: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<DebugHeadersResponse>, AppError> {
    let profiles = registry();

    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or("upload").to_string();
        let bytes = field.bytes().await?;
        let kind = detect_file_kind(&filename);

        let (_, _, file_hint) = parse_file(&bytes, kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let profile = detect_bank(&profiles, &file_hint);

        let (raw_rows, headers, _) = parse_file(&bytes, kind, profile)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let col = |aliases: &[&str]| -> serde_json::Value {
            for a in aliases {
                if let Some(i) = headers.iter().position(|h| h.contains(a)) {
                    return serde_json::json!({ "alias": a, "col_index": i, "header": &headers[i] });
                }
            }
            serde_json::json!(null)
        };

        let col_map = serde_json::json!({
            "txn_date":    col(profile.txn_date_aliases),
            "value_date":  col(profile.value_date_aliases),
            "description": col(profile.description_aliases),
            "debit":       col(profile.debit_aliases),
            "credit":      col(profile.credit_aliases),
            "balance":     col(profile.balance_aliases),
            "ref":         col(profile.ref_aliases),
        });

        let first_data_rows: Vec<Vec<String>> = raw_rows.iter().take(5)
            .map(|r| vec![
                r.txn_date.clone(),
                r.description.clone(),
                r.debit.map(|d| d.to_string()).unwrap_or_default(),
                r.credit.map(|c| c.to_string()).unwrap_or_default(),
            ])
            .collect();

        use uuid::Uuid;
        let sample_normalized =
            normalize(raw_rows, Uuid::nil(), Uuid::nil(), profile, "").len();

        return Ok(Json(DebugHeadersResponse {
            filename,
            bank_detected: profile.name.to_string(),
            header_row: headers,
            first_data_rows,
            col_map,
            sample_normalized,
        }));
    }
    Err(AppError::BadRequest("no file".into()))
}
