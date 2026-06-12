use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct CategoryRule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pattern: String,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleReq {
    pub pattern: String,
    pub category: String,
}
