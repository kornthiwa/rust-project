use axum::extract::rejection::JsonRejection;

use crate::application::error::AppError;
use crate::presentation::http::error::api_error::ApiError;

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::bad_request("invalid_json", rejection.body_text())
    }
}

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Validation { code, message } => {
                ApiError::bad_request(code, message.unwrap_or_default())
            }
            AppError::Unauthorized { code, message } => {
                ApiError::unauthorized(code, message.unwrap_or_default())
            }
            AppError::NotFound { code, message } => {
                ApiError::not_found(code, message.unwrap_or_default())
            }
            AppError::Conflict { code, message } => {
                ApiError::conflict(code, message.unwrap_or_default())
            }
            AppError::Internal {
                code,
                message,
                source,
            } => {
                tracing::error!(
                    app_error_code = code,
                    message = ?message,
                    source = ?source,
                    "internal application error"
                );
                ApiError::internal(code)
            }
        }
    }
}
