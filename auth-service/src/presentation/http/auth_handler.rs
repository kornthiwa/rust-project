use axum::{
    Json, Router,
    extract::State,
    extract::rejection::JsonRejection,
    http::StatusCode,
    routing::post,
};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::application::auth::service::LoginResult;
use crate::domain::account::entity::Account;
use crate::presentation::http::error::{ApiError, HttpResult};

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AccountResponse {
    id: u64,
    public_id: String,
    username: String,
    status: String,
    created_at: String,
}

impl AccountResponse {
    pub fn from_account(account: Account) -> Self {
        Self {
            id: account.id,
            public_id: account.public_id,
            username: account.username,
            status: account.status,
            created_at: account.created_at,
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(state): State<AppState>,
    payload: Result<Json<RegisterRequest>, JsonRejection>,
) -> HttpResult<(StatusCode, Json<AccountResponse>)> {
    let Json(payload) = payload.map_err(ApiError::from)?;
    let account = state
        .auth_service
        .register(payload.username, payload.password)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(AccountResponse::from_account(account)),
    ))
}

async fn login(
    State(state): State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> HttpResult<Json<LoginResult>> {
    let Json(payload) = payload.map_err(ApiError::from)?;
    let login_result = state
        .auth_service
        .login(payload.username, payload.password)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(login_result))
}
