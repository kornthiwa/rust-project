use axum::{
    Extension, Json, Router,
    extract::Query,
    extract::rejection::JsonRejection,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;

use crate::app::AppState;
use crate::application::message::service::CreateMessageInput;
use crate::domain::message::entity::Message;
use crate::presentation::http::error::{ApiError, HttpResult};
use crate::presentation::http::middleware::JwtClaims;

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub conversation_id: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_messages).post(create_message))
        .route("/{message_id}", get(get_message))
}

async fn list_messages(
    State(state): State<AppState>,
    Query(query): Query<ListMessagesQuery>,
) -> HttpResult<Json<Vec<Message>>> {
    let messages = state
        .message_service
        .list_messages(&query.conversation_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(messages))
}

async fn get_message(
    Path(message_id): Path<u64>,
    State(state): State<AppState>,
) -> HttpResult<Json<Message>> {
    let message = state
        .message_service
        .get_message(message_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(message))
}

async fn create_message(
    Extension(claims): Extension<JwtClaims>,
    State(state): State<AppState>,
    payload: Result<Json<CreateMessageInput>, JsonRejection>,
) -> HttpResult<(StatusCode, Json<Message>)> {
    let Json(input) = payload.map_err(ApiError::from)?;
    let message = state
        .message_service
        .create_message(claims.sub, input)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(message)))
}
