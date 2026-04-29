use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

use crate::app::AppState;
use crate::application::user::service::{CreateUserInput, UpdateUserInput};
use crate::domain::user::entity::User;
use crate::presentation::http::error::{ApiError, HttpResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route(
            "/{user_id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}

async fn list_users(State(state): State<AppState>) -> HttpResult<Json<Vec<User>>> {
    let users = state
        .user_service
        .list_users()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(users))
}

async fn get_user(
    Path(user_id): Path<u64>,
    State(state): State<AppState>,
) -> HttpResult<Json<User>> {
    let user = state
        .user_service
        .get_user(user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(user))
}

async fn create_user(
    State(state): State<AppState>,
    payload: Result<Json<CreateUserInput>, JsonRejection>,
) -> HttpResult<(StatusCode, Json<User>)> {
    let Json(input) = payload.map_err(ApiError::from)?;
    let user = state
        .user_service
        .create_user(input)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(
    Path(user_id): Path<u64>,
    State(state): State<AppState>,
    payload: Result<Json<UpdateUserInput>, JsonRejection>,
) -> HttpResult<Json<User>> {
    let Json(input) = payload.map_err(ApiError::from)?;
    let user = state
        .user_service
        .update_user(user_id, input)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(user))
}

async fn delete_user(
    Path(user_id): Path<u64>,
    State(state): State<AppState>,
) -> HttpResult<StatusCode> {
    state
        .user_service
        .delete_user(user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
