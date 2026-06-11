use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::Path, extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{config::Config, error::AppError, AppState};

use super::middleware::{AdminUser, CurrentUser};
use super::models::*;

pub async fn do_register(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: &str,
    cfg: &Config,
) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash error: {e}"))?
        .to_string();

    let user: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(email)
    .bind(&hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.code().as_deref() == Some("23505") {
                return anyhow::anyhow!("An account with that email already exists");
            }
        }
        anyhow::anyhow!("Registration failed — please try again")
    })?;

    issue_token(user.0, cfg)
}

pub async fn do_login(pool: &PgPool, email: &str, password: &str, cfg: &Config) -> anyhow::Result<String> {
    let row = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, created_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("invalid credentials"))?;

    let parsed = PasswordHash::new(&row.password_hash)
        .map_err(|e| anyhow::anyhow!("hash parse: {e}"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| anyhow::anyhow!("invalid credentials"))?;

    issue_token(row.id, cfg)
}

fn issue_token(user_id: uuid::Uuid, cfg: &Config) -> anyhow::Result<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("jwt: {e}"))
}

pub async fn setup_status_handler(
    State(state): State<AppState>,
) -> Json<SetupStatusResponse> {
    let admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    Json(SetupStatusResponse {
        setup_required: !admin_exists,
    })
}

pub async fn setup_handler(
    State(state): State<AppState>,
    Json(req): Json<SetupReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Database error".into()))?;

    if admin_exists {
        return Err(AppError::BadRequest("Setup already completed".into()));
    }

    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let token = do_register(&state.db, &req.email.trim(), &req.password, "admin", &state.config)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(AuthResponse { token }))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let token = do_login(&state.db, &req.email, &req.password, &state.config)
        .await
        .map_err(|_| AppError::Unauthorized)?;
    Ok(Json(AuthResponse { token }))
}

pub async fn me_handler(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<MeResponse>, AppError> {
    let row = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, created_at FROM users WHERE id = $1",
    )
    .bind(current_user.0)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::NotFound)?
    .ok_or(AppError::NotFound)?;

    Ok(Json(MeResponse {
        id: row.id,
        email: row.email,
        role: row.role,
    }))
}

pub async fn admin_create_user_handler(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(req): Json<AdminCreateUserReq>,
) -> Result<Json<UserResponse>, AppError> {
    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash error: {e}"))?
        .to_string();

    let user: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, 'user') RETURNING id",
    )
    .bind(&req.email)
    .bind(&hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.code().as_deref() == Some("23505") {
                return AppError::Conflict("A user with that email already exists".into());
            }
        }
        AppError::BadRequest("Failed to create user — please try again".into())
    })?;

    Ok(Json(UserResponse {
        id: user.0,
        email: req.email,
        role: "user".into(),
    }))
}

pub async fn list_users_handler(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let rows = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, created_at FROM users ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Failed to fetch users".into()))?;

    let users: Vec<UserResponse> = rows
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            email: u.email,
            role: u.role,
        })
        .collect();

    Ok(Json(users))
}

pub async fn reset_password_handler(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<ResetPasswordReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.new_password.len() < 8 {
        return Err(AppError::BadRequest("New password must be at least 8 characters".into()));
    }

    let row = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, created_at FROM users WHERE id = $1",
    )
    .bind(current_user.0)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::NotFound)?
    .ok_or(AppError::NotFound)?;

    let parsed = PasswordHash::new(&row.password_hash)
        .map_err(|_| AppError::BadRequest("Error processing password".into()))?;
    Argon2::default()
        .verify_password(req.current_password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|e| AppError::BadRequest(format!("Hash error: {e}")))?
        .to_string();

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_hash)
        .bind(current_user.0)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to update password".into()))?;

    Ok(Json(serde_json::json!({ "message": "Password updated successfully" })))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if admin.0 == user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".into()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::BadRequest("Failed to delete user".into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "message": "User deleted" })))
}

pub async fn register_handler(
    _state: State<AppState>,
    _req: Json<RegisterReq>,
) -> Result<Json<AuthResponse>, AppError> {
    Err(AppError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::sync::Arc;

    fn test_config() -> Config {
        Config {
            database_url: String::new(),
            ro_database_url: String::new(),
            jwt_secret: "test-secret-32-chars-min-aaaaaaaaaa".into(),
            claude_bin: "claude".into(),
            bind_addr: "127.0.0.1:0".into(),
            cors_origins: vec![],
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_register_and_login(pool: PgPool) {
        let cfg = test_config();

        let token = do_register(&pool, "test@test.com", "password123", "user", &cfg)
            .await
            .unwrap();
        assert!(!token.is_empty());

        let token2 = do_login(&pool, "test@test.com", "password123", &cfg)
            .await
            .unwrap();
        assert!(!token2.is_empty());

        let err = do_login(&pool, "test@test.com", "wrongpassword", &cfg).await;
        assert!(err.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_duplicate_email_rejected(pool: PgPool) {
        let cfg = test_config();
        do_register(&pool, "dup@test.com", "pass123", "user", &cfg).await.unwrap();
        let err = do_register(&pool, "dup@test.com", "pass456", "user", &cfg).await;
        assert!(err.is_err());
    }
}
