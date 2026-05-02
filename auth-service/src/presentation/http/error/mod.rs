pub mod api_error;
pub mod mappers;

pub use api_error::ApiError;

pub type HttpResult<T> = Result<T, ApiError>;
