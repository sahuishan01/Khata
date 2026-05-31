use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;

use crate::{config::Config, error::AppError, AppState};

use super::models::*;

pub async fn do_register(pool: &PgPool, email: &str, password: &str, cfg: &Config) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash error: {e}"))?
        .to_string();

    let user: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(email)
    .bind(&hash)
    .fetch_one(pool)
    .await
    .map_err(|_| anyhow::anyhow!("email already in use"))?;

    issue_token(user.0, cfg)
}

pub async fn do_login(pool: &PgPool, email: &str, password: &str, cfg: &Config) -> anyhow::Result<String> {
    let row = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash FROM users WHERE email = $1",
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

pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let token = do_register(&state.db, &req.email, &req.password, &state.config)
        .await
        .map_err(|e| AppError::Conflict(e.to_string()))?;
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

        let token = do_register(&pool, "test@test.com", "password123", &cfg)
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
        do_register(&pool, "dup@test.com", "pass123", &cfg).await.unwrap();
        let err = do_register(&pool, "dup@test.com", "pass456", &cfg).await;
        assert!(err.is_err());
    }
}
