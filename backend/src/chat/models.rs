use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub sql_used: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct AskReq {
    pub question: String,
}

#[derive(Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub sql_used: String,
}

#[derive(Deserialize)]
pub struct SqlGenOutput {
    pub sql: String,
    #[allow(dead_code)]
    pub explanation: String,
}
