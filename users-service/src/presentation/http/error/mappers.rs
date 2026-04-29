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
            AppError::Validation { code, message } => ApiError {
                status: axum::http::StatusCode::BAD_REQUEST,
                code,
                message,
            },
            AppError::NotFound { code, message } => ApiError {
                status: axum::http::StatusCode::NOT_FOUND,
                code,
                message,
            },
            AppError::Internal {
                code,
                message,
                source,
            } => {
                if let Some(source) = source {
                    tracing::error!(error = %source, code = code, "internal application error");
                } else if let Some(message) = message {
                    tracing::error!(error = %message, code = code, "internal application error");
                } else {
                    tracing::error!(code = code, "internal application error");
                }
                ApiError::internal(code)
            }
        }
    }
}
