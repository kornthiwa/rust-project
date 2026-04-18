use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, State, rejection::JsonRejection},
};

use crate::app::AppState;
use crate::application::auth::dto::{CreateAccountDto, LoginDto};
use crate::application::auth::error::AppError;
use crate::domain::auth::entity::{CreateAccountResult, LoginResult, MeResult};

pub async fn create_account_handler(
    State(state): State<Arc<AppState>>,
    dto: Result<Json<CreateAccountDto>, JsonRejection>,
) -> Result<Json<CreateAccountResult>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let result = state.auth_service.create_account(dto.into()).await?;
    Ok(Json(result))
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    dto: Result<Json<LoginDto>, JsonRejection>,
) -> Result<Json<LoginResult>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let result = state.auth_service.login(dto.into()).await?;
    Ok(Json(result))
}

pub async fn me_handler(
    Extension(me): Extension<MeResult>,
) -> Result<Json<MeResult>, AppError> {
    Ok(Json(me))
}
