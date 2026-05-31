use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::AppState;

use super::models::Claims;

#[derive(Clone, Debug)]
pub struct CurrentUser(pub uuid::Uuid);

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = decode::<Claims>(
            auth,
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
