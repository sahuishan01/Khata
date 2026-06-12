use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Budget {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category: String,
    pub monthly_limit: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetReq {
    pub category: String,
    pub monthly_limit: f64,
}

#[derive(Debug, Serialize)]
pub struct BudgetStatus {
    pub category: String,
    pub monthly_limit: f64,
    pub spent: f64,
    pub pct: f64,
}
