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
    pub is_transfer: bool,
    pub notes: String,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotesReq {
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTxnReq {
    pub txn_date: String,
    pub value_date: String,
    pub description: String,
    pub amount: f64,
    pub direction: String,
    pub category: String,
    pub bank_ref: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub direction: Option<String>,
    pub search: Option<String>,
    pub category: Option<String>,
    pub bank: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
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
    pub total_invested: f64,
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
    pub savings_rate_pct: f64,
    pub avg_daily_spend: f64,
    pub month_comparison: MonthComparison,
    pub largest_expense: Option<TxnRow>,
    pub total_transactions: i64,
    pub total_invested: f64,
}

// ── Analytics Explore ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub group_by: Option<String>,
    pub dimension: Option<String>,
    pub category: Option<String>,
    pub bank: Option<String>,
    pub direction: Option<String>,
    pub compare: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsSeries {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub comparison_values: Option<Vec<f64>>,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub series: AnalyticsSeries,
    pub insights: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DetailQuery {
    pub category: Option<String>,
    pub month: Option<String>,
    pub payee: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DetailResponse {
    pub total_spent: f64,
    pub total_earned: f64,
    pub txn_count: i64,
    pub trend: Vec<MonthBucket>,
    pub top_payees: Vec<TopMerchant>,
    pub top_categories: Vec<CategoryBucket>,
}

#[derive(Debug, Serialize)]
pub struct HighlightsResponse {
    pub highest_earning_month: Option<MonthBucket>,
    pub highest_spending_month: Option<MonthBucket>,
    pub biggest_expense: Option<TxnRow>,
    pub biggest_income: Option<TxnRow>,
    pub top_payee: Option<TopMerchant>,
    pub most_frequent_payee: Option<(String, i64)>,
    pub top_category: Option<CategoryBucket>,
    pub best_savings_month: Option<(String, f64)>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AccountBalance {
    pub bank: String,
    pub account_label: String,
    pub total_spent: f64,
    pub total_earned: f64,
    pub balance: f64,
    pub txn_count: i64,
}

#[derive(Debug, Serialize)]
pub struct RecurringTxn {
    pub description: String,
    pub amount: f64,
    pub frequency: String,
    pub months: i64,
    pub category: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SyncTxn {
    pub id: String,
    pub version: i32,
    pub category: Option<String>,
    pub notes: Option<String>,
    pub is_transfer: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub success: Vec<String>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Serialize)]
pub struct SyncConflict {
    pub id: String,
    pub local_version: i32,
    pub server_version: i32,
    pub server_txn: TxnRow,
}
