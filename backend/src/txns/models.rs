use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct TxnRow {
    pub id: Uuid,
    pub txn_date: NaiveDate,
    pub value_date: NaiveDate,
    pub description: String,
    pub amount: f64,
    pub direction: String,
    pub balance: Option<f64>,
    pub bank: String,
    pub bank_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub direction: Option<String>,
    pub search: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct TxnListResponse {
    pub data: Vec<TxnRow>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MonthBucket {
    pub month: String,
    pub spent: f64,
    pub earned: f64,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_spent: f64,
    pub total_earned: f64,
    pub net: f64,
    pub monthly: Vec<MonthBucket>,
    pub top_debits: Vec<TopMerchant>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TopMerchant {
    pub description: String,
    pub total: f64,
}
