use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    extract::rejection::JsonRejection,
    routing::get,
};

use crate::application::account::service::{CreateAccountInput, UpdateAccountInput};
use crate::app::AppState;
use crate::domain::account::entity::Account;
use crate::presentation::http::error::{ApiError, HttpResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_accounts).post(create_account))
        .route(
            "/{account_id}",
            get(get_account).put(update_account).delete(delete_account),
        )
}

async fn list_accounts(State(state): State<AppState>) -> HttpResult<Json<Vec<Account>>> {
    let accounts = state
        .account_service
        .list_accounts()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(accounts))
}

async fn get_account(
    Path(account_id): Path<u64>,
    State(state): State<AppState>,
) -> HttpResult<Json<Account>> {
    let account = state
        .account_service
        .get_account(account_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("account_not_found", "account not found"))?;

    Ok(Json(account))
}

async fn create_account(
    State(state): State<AppState>,
    payload: Result<Json<CreateAccountInput>, JsonRejection>,
) -> HttpResult<(StatusCode, Json<Account>)> {
    let Json(input) = payload.map_err(ApiError::from)?;
    let account = state
        .account_service
        .create_account(input)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(account)))
}

async fn update_account(
    Path(account_id): Path<u64>,
    State(state): State<AppState>,
    payload: Result<Json<UpdateAccountInput>, JsonRejection>,
) -> HttpResult<Json<Account>> {
    let Json(input) = payload.map_err(ApiError::from)?;
    let account = state
        .account_service
        .update_account(account_id, input)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("account_not_found", "account not found"))?;

    Ok(Json(account))
}

async fn delete_account(
    Path(account_id): Path<u64>,
    State(state): State<AppState>,
) -> HttpResult<StatusCode> {
    match state.account_service.delete_account(account_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(ApiError::not_found("account_not_found", "account not found")),
        Err(error) => Err(ApiError::from(error)),
    }
}
