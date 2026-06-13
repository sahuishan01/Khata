use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::AppState;

use super::models::{Claims, Role};

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
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth = token_from_parts(parts).ok_or(StatusCode::UNAUTHORIZED)?;

        let token = decode::<Claims>(
            &auth,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let uid = token
            .claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

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
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let uid = token
            .claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let row = sqlx::query_as::<_, super::models::User>(
            "SELECT id, email, password_hash, role, created_at FROM users WHERE id = $1",
        )
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

        if Role::from_str(&row.role) != Role::Admin {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(AdminUser(uid))
    }
}
