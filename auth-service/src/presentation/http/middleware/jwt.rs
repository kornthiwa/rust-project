use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::app::AppState;
use crate::presentation::http::error::ApiError;

pub async fn jwt_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::unauthorized("missing_authorization", "missing Authorization header")
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("invalid_authorization", "expected Bearer token"))?;

    let claims = state
        .auth_service
        .verify_jwt(token)
        .map_err(ApiError::from)?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
