use axum::{extract::{Multipart, State}, Json};
use sha2::{Digest, Sha256};

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::{
    detect::{detect_bank, detect_file_kind},
    models::UploadResponse,
    normalize::normalize,
    parse::parse_file,
    profiles::registry,
    store::{insert_statement, store_transactions},
};

pub async fn upload_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let profiles = registry();

    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or("upload").to_string();
        let bytes = field.bytes().await?;

        // Reject re-upload of identical file (by SHA-256)
        let sha = hex::encode(Sha256::digest(&bytes));
        let exists: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM statements WHERE user_id = $1 AND file_sha256 = $2",
        )
        .bind(user_id)
        .bind(&sha)
        .fetch_optional(&state.db)
        .await?;
        if exists.is_some() {
            return Err(AppError::Conflict(format!(
                "{filename} has already been imported"
            )));
        }

        let kind = detect_file_kind(&filename);

        // First pass with generic profile to detect headers and bank
        let (_, headers) = parse_file(&bytes, kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let header_strs: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        let profile = detect_bank(&profiles, &header_strs, "");

        // Second pass with correct profile
        let (raw_rows, _) = parse_file(&bytes, kind, profile)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let rows_parsed = raw_rows.len();
        if rows_parsed == 0 {
            return Err(AppError::BadRequest(
                "No transaction rows found in file".into(),
            ));
        }

        let stmt_id = insert_statement(
            &state.db,
            user_id,
            profile.name,
            &filename,
            &sha,
            rows_parsed as i32,
        )
        .await?;

        let txns = normalize(raw_rows, user_id, stmt_id, profile, "default");
        let (inserted, skipped_duplicates) =
            store_transactions(&state.db, user_id, &txns).await?;

        return Ok(Json(UploadResponse {
            bank_detected: profile.name.to_string(),
            rows_parsed,
            inserted,
            skipped_duplicates,
        }));
    }

    Err(AppError::BadRequest("No file in upload".into()))
}
