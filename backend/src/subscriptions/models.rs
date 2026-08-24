use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub billing_cycle: String,
    pub next_due_date: NaiveDate,
    pub category: String,
    pub auto_detected: bool,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionReq {
    pub name: String,
    pub amount: f64,
    pub billing_cycle: String,
    pub next_due_date: NaiveDate,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionReq {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub billing_cycle: Option<String>,
    pub next_due_date: Option<NaiveDate>,
    pub active: Option<bool>,
}
