use axum::{
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

#[derive(Debug, Deserialize)]
pub struct ReportQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaxSummaryResponse {
    pub total_80c_eligible: f64,
    pub total_80d_medical: f64,
    pub total_charity_80g: f64,
    pub breakdown: Vec<TaxItem>,
}

#[derive(Debug, Serialize)]
pub struct TaxItem {
    pub section: String,
    pub description: String,
    pub amount: f64,
    pub txn_date: String,
}

pub async fn export_csv_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Query(params): Query<ReportQueryParams>,
) -> Result<Response, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<(String, String, f64, String, String, String)> = sqlx::query_as(
        "SELECT txn_date::text, description, amount::double precision, direction, bank, category \
         FROM transactions WHERE user_id = $1 AND deleted = false ORDER BY txn_date DESC"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    let mut csv_out = String::from("Date,Description,Amount,Direction,Bank,Category\n");
    for (date, desc, amt, dir, bank, cat) in rows {
        let clean_desc = desc.replace('"', "\"\"");
        csv_out.push_str(&format!(
            "\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\"\n",
            date, clean_desc, amt, dir, bank, cat
        ));
    }

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"khata_transactions_report.csv\"",
            ),
        ],
        csv_out,
    )
        .into_response())
}

pub async fn tax_summary_handler(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<TaxSummaryResponse>, AppError> {
    let mut tx = state.db.begin().await?;
    crate::db::set_current_user(&mut *tx, user_id).await?;

    let rows: Vec<(String, f64, String, String)> = sqlx::query_as(
        "SELECT txn_date::text, amount::double precision, description, category \
         FROM transactions WHERE user_id = $1 AND deleted = false \
         AND (LOWER(category) LIKE '%insurance%' OR LOWER(category) LIKE '%investment%' \
              OR LOWER(category) LIKE '%medical%' OR LOWER(category) LIKE '%tax%')"
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    let mut total_80c = 0.0;
    let mut total_80d = 0.0;
    let mut total_80g = 0.0;
    let mut breakdown = Vec::new();

    for (date, amt, desc, cat) in rows {
        let cat_lower = cat.to_lowercase();
        let sec = if cat_lower.contains("insurance") || cat_lower.contains("medical") {
            total_80d += amt;
            "Section 80D (Health Insurance & Medical)"
        } else if cat_lower.contains("charity") || cat_lower.contains("donation") {
            total_80g += amt;
            "Section 80G (Donations)"
        } else {
            total_80c += amt;
            "Section 80C (Investments & PPF/ELSS)"
        };

        breakdown.push(TaxItem {
            section: sec.to_string(),
            description: desc,
            amount: amt,
            txn_date: date,
        });
    }

    Ok(Json(TaxSummaryResponse {
        total_80c_eligible: total_80c,
        total_80d_medical: total_80d,
        total_charity_80g: total_80g,
        breakdown,
    }))
}
