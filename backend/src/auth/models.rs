use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            _ => Role::User,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub must_reset_password: bool,
    pub password_changed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn role_enum(&self) -> Role {
        Role::from_str(&self.role)
    }
}

#[derive(Deserialize)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub must_reset_password: bool,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub setup_required: bool,
}

#[derive(Deserialize)]
pub struct SetupReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AdminCreateUserReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordReq {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
