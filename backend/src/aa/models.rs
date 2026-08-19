use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AaSettingsResponse {
    pub auto_fetch_enabled: bool,
    pub fetch_interval_days: i32,
    pub last_fetched_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_fetch_due_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAaSettingsReq {
    pub auto_fetch_enabled: bool,
    pub fetch_interval_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitConsentResponse {
    pub consent_id: String,
    pub consent_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentStatusResponse {
    pub consent_id: String,
    pub status: String,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub message: String,
    pub transactions_added: usize,
    pub portfolio_assets_updated: usize,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AaConsentRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub consent_id: String,
    pub handle_id: String,
    pub status: String,
    pub fi_types: Vec<String>,
    pub valid_until: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
