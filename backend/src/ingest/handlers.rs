use axum::{
    extract::{Multipart, State},
    Json,
};
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

        // All DB work in one transaction so SET LOCAL covers every query
        let mut tx = state.db.begin().await?;
        sqlx::query(&format!("SET LOCAL app.current_user_id = '{user_id}'"))
            .execute(&mut *tx)
            .await?;

        // Reject re-upload of identical file
        let exists: Option<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM statements WHERE user_id = $1 AND file_sha256 = $2",
        )
        .bind(user_id)
        .bind(&sha)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_some() {
            tx.rollback().await?;
            return Err(AppError::Conflict(format!(
                "{filename} has already been imported"
            )));
        }

        let kind = detect_file_kind(&filename);

        // First pass with generic profile to discover headers and detect bank
        let (_, headers) = parse_file(&bytes, kind.clone(), profiles.last().unwrap())
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let header_strs: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        let profile = detect_bank(&profiles, &header_strs, "");

        // Second pass with the matched profile
        let (raw_rows, _) = parse_file(&bytes, kind, profile)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let rows_parsed = raw_rows.len();
        if rows_parsed == 0 {
            tx.rollback().await?;
            return Err(AppError::BadRequest(
                "No transaction rows found in file".into(),
            ));
        }

        // Insert statement record (within same transaction / RLS context)
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

        // store_transactions starts its own transaction with SET LOCAL
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
