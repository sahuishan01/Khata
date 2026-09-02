use axum::{
    async_trait,
    extract::{FromRequestParts, OriginalUri},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::json;

use crate::AppState;

use super::models::{Claims, Role};

/// Full request paths (as seen before router nesting strips the prefix) that a
/// user whose `must_reset_password` flag is still set is allowed to reach.
/// Everything else is rejected with 403 until the password has been changed.
const PASSWORD_RESET_ALLOWED_PATHS: [&str; 2] = ["/api/auth/reset-password", "/api/auth/me"];

fn password_reset_required_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "password_reset_required" })),
    )
        .into_response()
}

fn jwt_validation() -> Validation {
    let mut v = Validation::new(Algorithm::HS256);
    v.set_issuer(&["khata"]);
    v.set_audience(&["khata-api"]);
    v
}

fn token_from_parts(parts: &Parts) -> Option<String> {
    if let Some(auth) = parts
        .headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(auth.to_string());
    }
    if let Some(cookie) = parts
        .headers
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
    {
        for pair in cookie.split(';') {
            let pair = pair.trim();
            if let Some(value) = pair.strip_prefix("khata_token=") {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct CurrentUser(pub uuid::Uuid);

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth = token_from_parts(parts).ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

        let token = decode::<Claims>(
            &auth,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &jwt_validation(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

        let uid = token
            .claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

        // Verify token version hasn't been revoked, and load the reset flag.
        let row: Option<(i32, bool)> = sqlx::query_as(
            "SELECT token_version, must_reset_password FROM users WHERE id = $1",
        )
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

        let must_reset_password = match row {
            Some((db_ver, must_reset)) if db_ver == token.claims.ver => must_reset,
            _ => return Err(StatusCode::UNAUTHORIZED.into_response()),
        };

        // Enforce mandatory password reset server-side. A user whose
        // `must_reset_password` flag is still set (e.g. an admin-created account)
        // may only reach the password-change endpoint and its own profile until
        // the password has actually been changed; every other request is 403.
        if must_reset_password {
            let path = parts
                .extensions
                .get::<OriginalUri>()
                .map(|u| u.0.path())
                .unwrap_or_else(|| parts.uri.path());
            if !PASSWORD_RESET_ALLOWED_PATHS.contains(&path) {
                return Err(password_reset_required_response());
            }
        }

        Ok(CurrentUser(uid))
    }
}

#[derive(Clone, Debug)]
pub struct AdminUser(pub uuid::Uuid);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth = token_from_parts(parts).ok_or(StatusCode::UNAUTHORIZED)?;

        let token = decode::<Claims>(
            &auth,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &jwt_validation(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let uid = token
            .claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let row = sqlx::query_as::<_, super::models::User>(
            "SELECT id, email, password_hash, role, created_at, must_reset_password, password_changed_at, token_version FROM users WHERE id = $1",
        )
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

        if Role::from_str(&row.role) != Role::Admin {
            return Err(StatusCode::FORBIDDEN);
        }

        if row.token_version != token.claims.ver {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(AdminUser(uid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::handlers::do_register;
    use crate::config::Config;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use sqlx::PgPool;

    fn test_config() -> Config {
        Config {
            database_url: String::new(),
            ro_database_url: String::new(),
            jwt_secret: "test-secret-32-chars-min-aaaaaaaaaa".into(),
            claude_bin: "claude".into(),
            bind_addr: "127.0.0.1:0".into(),
            cors_origins: vec![],
            cookie_secure: true,
            allow_remote_setup: false,
        }
    }

    fn test_state(pool: &PgPool) -> AppState {
        AppState {
            db: pool.clone(),
            db_ro: pool.clone(),
            config: Arc::new(test_config()),
            chat_ratelimit: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Build request `Parts` the way the axum router would: with the full
    /// request path recorded in the `OriginalUri` extension (nesting strips the
    /// prefix from `parts.uri`, so the extractor relies on `OriginalUri`).
    fn parts_for(uri: &str, token: &str) -> Parts {
        let req = axum::http::Request::builder()
            .uri(uri)
            .header("Authorization", format!("Bearer {token}"))
            .extension(OriginalUri(uri.parse().unwrap()))
            .body(())
            .unwrap();
        req.into_parts().0
    }

    async fn set_must_reset(pool: &PgPool, email: &str, value: bool) {
        sqlx::query("UPDATE users SET must_reset_password = $1 WHERE email = $2")
            .bind(value)
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn must_reset_user_blocked_from_normal_routes(pool: PgPool) {
        let cfg = test_config();
        let token = do_register(&pool, "reset_block@test.com", "password12345", "user", &cfg)
            .await
            .unwrap();
        set_must_reset(&pool, "reset_block@test.com", true).await;
        let state = test_state(&pool);

        let mut parts = parts_for("/api/txns", &token);
        let rejection = CurrentUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("must_reset_password user must be rejected on protected routes");

        assert_eq!(rejection.status(), StatusCode::FORBIDDEN);
        let body = axum::body::to_bytes(rejection.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "password_reset_required");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn must_reset_user_allowed_on_reset_and_me(pool: PgPool) {
        let cfg = test_config();
        let token = do_register(&pool, "reset_allow@test.com", "password12345", "user", &cfg)
            .await
            .unwrap();
        set_must_reset(&pool, "reset_allow@test.com", true).await;
        let state = test_state(&pool);

        for uri in ["/api/auth/reset-password", "/api/auth/me"] {
            let mut parts = parts_for(uri, &token);
            assert!(
                CurrentUser::from_request_parts(&mut parts, &state).await.is_ok(),
                "must_reset_password user must still reach {uri}"
            );
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn normal_user_not_affected(pool: PgPool) {
        let cfg = test_config();
        let token = do_register(&pool, "normal@test.com", "password12345", "user", &cfg)
            .await
            .unwrap();
        // must_reset_password defaults to false for do_register.
        let state = test_state(&pool);

        let mut parts = parts_for("/api/txns", &token);
        assert!(
            CurrentUser::from_request_parts(&mut parts, &state).await.is_ok(),
            "user without must_reset_password must be unaffected"
        );
    }
}
