mod api_error;
mod mappers;

pub use api_error::ApiError;

pub type HttpResult<T> = Result<T, ApiError>;
