use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Category editing ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryReq {
    pub category: String,
    /// How broadly to apply the change
    pub scope: UpdateScope,
    /// Required when scope is Contains; the substring to match in description
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateScope {
    /// Only this transaction
    Single,
    /// All transactions with the exact same description
    SameDescription,
    /// All transactions whose description contains `keyword`
    Contains,
}

#[derive(Debug, Serialize)]
pub struct UpdateCategoryResponse {
    pub updated: u64,
}

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
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub direction: Option<String>,
    pub search: Option<String>,
    pub category: Option<String>,
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

// ── Dashboard ─────────────────────────────────────────────────────────────────

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

// ── Analysis ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CategoryBucket {
    pub category: String,
    pub amount: f64,
    pub txn_count: i64,
    pub pct: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MonthComparison {
    pub this_month: f64,
    pub last_month: f64,
    pub change_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct AnalysisStats {
    pub category_breakdown: Vec<CategoryBucket>,
    pub savings_rate_pct: f64,        // (earned - spent) / earned * 100
    pub avg_daily_spend: f64,         // total_spent / distinct days with debits
    pub month_comparison: MonthComparison,
    pub largest_expense: Option<TxnRow>,
    pub total_transactions: i64,
}
