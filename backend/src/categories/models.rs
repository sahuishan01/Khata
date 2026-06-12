use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub txn_type: String,
    pub color: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryReq {
    pub name: String,
    pub txn_type: String,
    pub color: Option<String>,
    pub description: Option<String>,
}
