use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GoalRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub target_amount: f64,
    pub current_amount: f64,
    pub target_date: Option<NaiveDate>,
    pub color_hex: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGoalReq {
    pub name: String,
    pub target_amount: f64,
    pub current_amount: Option<f64>,
    pub target_date: Option<NaiveDate>,
    pub color_hex: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGoalProgressReq {
    pub current_amount: f64,
}
