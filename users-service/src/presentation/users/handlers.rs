use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
};

use crate::app::AppState;
use crate::application::users::dto::{CreateUserDto, UpdateUserDto};
use crate::application::users::error::AppError;
use crate::domain::users::entity::User;

pub async fn list_users_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = state.user_service.list_users().await?;
    Ok(Json(users))
}

pub async fn get_user_by_id_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(&id).await?;
    Ok(Json(user))
}

pub async fn create_user_handler(
    State(state): State<Arc<AppState>>,
    dto: Result<Json<CreateUserDto>, JsonRejection>,
) -> Result<Json<User>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let user = state.user_service.create_user(dto).await?;
    Ok(Json(user))
}

pub async fn update_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    dto: Result<Json<UpdateUserDto>, JsonRejection>,
) -> Result<Json<User>, AppError> {
    let Json(dto) = dto.map_err(AppError::from)?;
    let user = state.user_service.update_user(&id, dto).await?;
    Ok(Json(user))
}

pub async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.delete_user(&id).await?;
    Ok(Json(user))
}