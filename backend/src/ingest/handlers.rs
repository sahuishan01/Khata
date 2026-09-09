use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{audit, auth::middleware::CurrentUser, error::AppError, AppState};

use super::{
    detect::{detect_bank, detect_file_kind},
    models::UploadResponse,
    normalize::normalize,
    parse::parse_file,
    profiles::registry,
    store::store_transactions,
};

// ── Upload validation ──────────────────────────────────────────────────────────

const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;
const ALLOWED_EXTENSIONS: [&str; 4] = ["csv", "xls", "xlsx", "pdf"];

fn validate_upload(filename: &str, bytes: &[u8]) -> Result<(), AppError> {
    if bytes.len() > MAX_UPLOAD_SIZE {
        return Err(AppError::BadRequest(format!(
            "File too large: {} bytes (maximum {})",
            bytes.len(),
            MAX_UPLOAD_SIZE
        )));
    }

    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file extension: .{} (allowed: .csv, .xls, .xlsx, .pdf)",
            ext
        )));
    }

    match ext.as_str() {
        "pdf" => {
            if bytes.len() < 4 || &bytes[..4] != b"%PDF" {
                return Err(AppError::BadRequest(
                    "Invalid PDF file: missing %PDF header".into(),
                ));
            }
        }
        "xlsx" => {
            if bytes.len() < 4 || bytes[..4] != [0x50, 0x4B, 0x03, 0x04] {
                return Err(AppError::BadRequest(
                    "Invalid XLSX file: missing PK zip header (0x50 0x4B 0x03 0x04)".into(),
                ));
            }
        }
        "xls" => {
            if bytes.len() < 4 || bytes[..4] != [0xD0, 0xCF, 0x11, 0xE0] {
                return Err(AppError::BadRequest(
                    "Invalid XLS file: missing DCF header (0xD0 0xCF 0x11 0xE0)".into(),
                ));
            }
        }
        "csv" => {
            // No magic bytes for CSV – rely on extension validation
        }
        _ => unreachable!(), // validated above
    }

    Ok(())
}

// ── Upload ────────────────────────────────────────────────────────────────────

pub async fn upload_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let profiles = registry();

    // Collect the multipart fields: one file plus optional `password` /
    // `save_password` text fields for encrypted statements.
    let mut filename = String::new();
    let mut raw: Vec<u8> = Vec::new();
    let mut password: Option<String> = None;
    let mut save_password = false;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(str::to_string);
        let file_name = field.file_name().map(str::to_string);
        match name.as_deref() {
            Some("password") => password = Some(field.text().await?),
            Some("save_password") => {
                save_password = matches!(field.text().await?.as_str(), "true" | "1" | "on")
            }
            _ => {
                filename = file_name.unwrap_or_else(|| "upload".to_string());
                raw = field.bytes().await?.to_vec();
            }
        }
    }

    let password = password.filter(|p| !p.trim().is_empty());

    if raw.is_empty() {
        return Err(AppError::BadRequest("No file in upload".into()));
    }

    validate_upload(&filename, &raw)?;

    let kind = detect_file_kind(&filename);

    {
        // Transparently decrypt password-protected statement PDFs. Try, in order:
        // a password supplied with this upload, then the user's stored one.
        let bytes: std::borrow::Cow<'_, [u8]> = if kind == super::detect::FileKind::Pdf
            && super::email::pdf_decrypt::is_encrypted(&raw)
        {
            let stored_pw: Option<String> = {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT encrypted_pdf_password FROM user_email_configs \
                     WHERE user_id = $1 AND encrypted_pdf_password IS NOT NULL AND encrypted_pdf_password <> ''",
                )
                .bind(user_id)
                .fetch_optional(&state.db)
                .await?;
                match row {
                    Some((enc,)) => Some(
                        super::crypto::decrypt_credential(
                            &enc, &state.config.jwt_secret, &user_id.to_string(),
                        )
                        .map_err(|_| AppError::BadRequest("Could not read stored statement password".into()))?,
                    ),
                    None => None,
                }
            };

            let candidates: Vec<(String, bool)> = password
                .iter()
                .cloned()
                .map(|p| (p, true))
                .chain(stored_pw.into_iter().map(|p| (p, false)))
                .collect();

            if candidates.is_empty() {
                return Err(AppError::UploadPassword { incorrect: false });
            }

            let mut decrypted: Option<(Vec<u8>, String)> = None;
            for (pw, _) in &candidates {
                if let Ok(plain) = super::email::pdf_decrypt::decrypt_pdf(&raw, pw) {
                    decrypted = Some((plain, pw.clone()));
                    break;
                }
            }
            let (plain, working_pw) = decrypted
                .ok_or(AppError::UploadPassword { incorrect: true })?;

            // Persist the password for next time (uploads + email sync) when asked.
            if save_password {
                if let Ok(enc) = super::crypto::encrypt_credential(
                    &working_pw, &state.config.jwt_secret, &user_id.to_string(),
                ) {
                    let _ = sqlx::query(
                        "UPDATE user_email_configs SET encrypted_pdf_password = $1 WHERE user_id = $2",
                    )
                    .bind(&enc)
                    .bind(user_id)
                    .execute(&state.db)
                    .await;
                }
            }

            std::borrow::Cow::Owned(plain)
        } else {
            std::borrow::Cow::Borrowed(&raw[..])
        };

        let sha = hex::encode(Sha256::digest(bytes.as_ref()));

        let mut tx = state.db.begin().await?;
        crate::db::set_current_user(&mut *tx, user_id).await?;

        // First pass: generic profile → get full file hint text for bank detection
        let (_, _, file_hint, _) = parse_file(bytes.as_ref(), kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let profile = detect_bank(&profiles, &file_hint);

        // Second pass: correct profile
        let (raw_rows, _, _, low_conf) = parse_file(bytes.as_ref(), kind, profile)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let warnings: Vec<String> = low_conf.into_iter().collect();

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
                warnings: warnings.clone(),
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
            warnings,
        }))
    }
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

        validate_upload(&filename, &bytes)?;

        let kind = detect_file_kind(&filename);

        let (_, _, file_hint, _) = parse_file(bytes.as_ref(), kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let profile = detect_bank(&profiles, &file_hint);

        let (raw_rows, headers, _, _) = parse_file(bytes.as_ref(), kind, profile)
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

#[derive(Deserialize)]
pub struct ClearDataReq {
    pub confirm: bool,
}

pub async fn clear_all_data_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<ClearDataReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !req.confirm {
        return Err(AppError::BadRequest("confirm must be true".into()));
    }

    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let deleted = sqlx::query("DELETE FROM transactions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete transactions".into()))?;

    sqlx::query("DELETE FROM statements WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete statements".into()))?;

    sqlx::query("DELETE FROM chat_messages WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .ok();

    // Invalidate all existing tokens for this user
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .ok();

    tx.commit().await?;

    audit::log_audit(
        &state.db,
        user_id,
        "clear_all_data",
        Some(serde_json::json!({"deleted_transactions": deleted.rows_affected()})),
    ).await;

    Ok(Json(serde_json::json!({
        "message": "All data cleared",
        "deleted_transactions": deleted.rows_affected()
    })))
}
