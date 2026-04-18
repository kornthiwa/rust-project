use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::domain::auth::error::DomainError;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Conflict,
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Internal,
}

impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::NotFound => AppError::NotFound,
            DomainError::Conflict => AppError::Conflict,
            DomainError::InvalidCredentials => AppError::Unauthorized,
            DomainError::InactiveAccount => AppError::Forbidden,
            DomainError::RepositoryFailure => AppError::Internal,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found".to_string(),
                "Resource not found.".to_string(),
            ),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                "conflict".to_string(),
                "Resource conflict.".to_string(),
            ),
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                "bad_request".to_string(),
                message,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized".to_string(),
                "Missing or invalid access token.".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden".to_string(),
                "Access denied.".to_string(),
            ),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error".to_string(),
                "Unexpected internal error.".to_string(),
            ),
        };

        let body = ErrorResponse {
            status: status.as_u16(),
            error,
            message,
        };

        (status, Json(body)).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    status: u16,
    error: String,
    message: String,
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        AppError::BadRequest(format!(
            "Invalid JSON request body: {}",
            value.body_text()
        ))
    }
}
