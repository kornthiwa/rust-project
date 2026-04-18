use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;

use crate::app::AppState;
use crate::application::products::error::AppError;

#[derive(Debug, Clone, Deserialize)]
struct AccessTokenClaims {
    sub: String,
    username: String,
    exp: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub subject: String,
    pub username: String,
    pub expires_at_unix: usize,
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let authorization = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let token_data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let claims = token_data.claims;
    request.extensions_mut().insert(AuthContext {
        subject: claims.sub,
        username: claims.username,
        expires_at_unix: claims.exp,
    });

    Ok(next.run(request).await)
}
