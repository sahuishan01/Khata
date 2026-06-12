use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::middleware::CurrentUser, error::AppError, AppState};

use super::models::*;

pub async fn list_assets(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<PortfolioAsset>>, AppError> {
    let rows = sqlx::query_as::<_, PortfolioAsset>(
        "SELECT id, user_id, name, asset_type, value::float8, recorded_at::text FROM portfolio_assets WHERE user_id = $1 ORDER BY recorded_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch assets".into()))?;
    Ok(Json(rows))
}

pub async fn create_asset(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateAssetReq>,
) -> Result<Json<PortfolioAsset>, AppError> {
    if req.name.trim().is_empty() || req.value < 0.0 {
        return Err(AppError::BadRequest("Valid name and value required".into()));
    }
    let valid_types = ["bank", "mutual_fund", "stock", "fd", "cash", "other"];
    if !valid_types.contains(&req.asset_type.as_str()) {
        return Err(AppError::BadRequest("Invalid asset_type".into()));
    }

    let recorded = req.recorded_at.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let row = sqlx::query_as::<_, PortfolioAsset>(
        "INSERT INTO portfolio_assets (user_id, name, asset_type, value, recorded_at)
         VALUES ($1, $2, $3, $4, $5::date)
         RETURNING id, user_id, name, asset_type, value::float8, recorded_at::text",
    )
    .bind(user_id).bind(req.name.trim()).bind(&req.asset_type).bind(req.value).bind(&recorded)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to create asset".into()))?;
    Ok(Json(row))
}

pub async fn delete_asset(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("DELETE FROM portfolio_assets WHERE id = $1 AND user_id = $2")
        .bind(asset_id).bind(user_id)
        .execute(&state.db).await
        .map_err(|_| AppError::BadRequest("Failed".into()))?;
    if r.rows_affected() == 0 { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "message": "Asset deleted" })))
}

pub async fn list_liabilities(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<PortfolioLiability>>, AppError> {
    let rows = sqlx::query_as::<_, PortfolioLiability>(
        "SELECT id, user_id, name, liability_type, value::float8, recorded_at::text FROM portfolio_liabilities WHERE user_id = $1 ORDER BY recorded_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch liabilities".into()))?;
    Ok(Json(rows))
}

pub async fn create_liability(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(req): Json<CreateLiabilityReq>,
) -> Result<Json<PortfolioLiability>, AppError> {
    if req.name.trim().is_empty() || req.value < 0.0 {
        return Err(AppError::BadRequest("Valid name and value required".into()));
    }
    let valid_types = ["loan", "credit_card", "other"];
    if !valid_types.contains(&req.liability_type.as_str()) {
        return Err(AppError::BadRequest("Invalid liability_type".into()));
    }

    let recorded = req.recorded_at.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let row = sqlx::query_as::<_, PortfolioLiability>(
        "INSERT INTO portfolio_liabilities (user_id, name, liability_type, value, recorded_at)
         VALUES ($1, $2, $3, $4, $5::date)
         RETURNING id, user_id, name, liability_type, value::float8, recorded_at::text",
    )
    .bind(user_id).bind(req.name.trim()).bind(&req.liability_type).bind(req.value).bind(&recorded)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to create liability".into()))?;
    Ok(Json(row))
}

pub async fn delete_liability(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(liability_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("DELETE FROM portfolio_liabilities WHERE id = $1 AND user_id = $2")
        .bind(liability_id).bind(user_id)
        .execute(&state.db).await
        .map_err(|_| AppError::BadRequest("Failed".into()))?;
    if r.rows_affected() == 0 { return Err(AppError::NotFound); }
    Ok(Json(serde_json::json!({ "message": "Liability deleted" })))
}

pub async fn net_worth_snapshot(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<NetWorthSnapshot>, AppError> {
    let assets = sqlx::query_as::<_, PortfolioAsset>(
        "SELECT id, user_id, name, asset_type, value::float8, recorded_at::text FROM portfolio_assets WHERE user_id = $1 ORDER BY recorded_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed".into()))?;

    let liabilities = sqlx::query_as::<_, PortfolioLiability>(
        "SELECT id, user_id, name, liability_type, value::float8, recorded_at::text FROM portfolio_liabilities WHERE user_id = $1 ORDER BY recorded_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed".into()))?;

    let total_assets: f64 = assets.iter().map(|a| a.value).sum();
    let total_liabilities: f64 = liabilities.iter().map(|l| l.value).sum();

    Ok(Json(NetWorthSnapshot {
        total_assets,
        total_liabilities,
        net_worth: total_assets - total_liabilities,
        assets,
        liabilities,
    }))
}

pub async fn net_worth_trend(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<NetWorthTrendPoint>>, AppError> {
    let asset_rows: Vec<(String, f64)> = sqlx::query_as(
        "SELECT recorded_at::text, SUM(value)::float8 FROM portfolio_assets WHERE user_id = $1 GROUP BY recorded_at ORDER BY recorded_at",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed".into()))?;

    let liability_rows: Vec<(String, f64)> = sqlx::query_as(
        "SELECT recorded_at::text, SUM(value)::float8 FROM portfolio_liabilities WHERE user_id = $1 GROUP BY recorded_at ORDER BY recorded_at",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed".into()))?;

    // Merge by date
    use std::collections::HashMap;
    let mut by_date: HashMap<String, f64> = HashMap::new();
    for (d, v) in &asset_rows { *by_date.entry(d.clone()).or_insert(0.0) += v; }
    for (d, v) in &liability_rows { *by_date.entry(d.clone()).or_insert(0.0) -= v; }

    let mut trend: Vec<NetWorthTrendPoint> = by_date
        .into_iter()
        .map(|(date, net_worth)| NetWorthTrendPoint { date, net_worth })
        .collect();
    trend.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(Json(trend))
}
