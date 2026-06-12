use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct UserAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub identifier: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountReq {
    pub label: String,
    pub identifier: String,
}
