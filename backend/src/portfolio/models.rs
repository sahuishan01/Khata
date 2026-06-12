use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct PortfolioAsset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub value: f64,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetReq {
    pub name: String,
    pub asset_type: String,
    pub value: f64,
    pub recorded_at: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct PortfolioLiability {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub liability_type: String,
    pub value: f64,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLiabilityReq {
    pub name: String,
    pub liability_type: String,
    pub value: f64,
    pub recorded_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NetWorthSnapshot {
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub net_worth: f64,
    pub assets: Vec<PortfolioAsset>,
    pub liabilities: Vec<PortfolioLiability>,
}

#[derive(Debug, Serialize)]
pub struct NetWorthTrendPoint {
    pub date: String,
    pub net_worth: f64,
}
