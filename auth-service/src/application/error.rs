#[derive(Debug)]
pub enum AppError {
    Validation {
        code: &'static str,
        message: Option<String>,
    },
    Unauthorized {
        code: &'static str,
        message: Option<String>,
    },
    NotFound {
        code: &'static str,
        message: Option<String>,
    },
    Conflict {
        code: &'static str,
        message: Option<String>,
    },
    Internal {
        code: &'static str,
        message: Option<String>,
        source: Option<String>,
    },
}

impl AppError {
    pub fn validation(code: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            code,
            message: Some(message.into()),
        }
    }

    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::Unauthorized {
            code,
            message: Some(message.into()),
        }
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::NotFound {
            code,
            message: Some(message.into()),
        }
    }

    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::Conflict {
            code,
            message: Some(message.into()),
        }
    }

    pub fn internal(code: &'static str) -> Self {
        Self::Internal {
            code,
            message: None,
            source: None,
        }
    }

    pub fn internal_with_source(code: &'static str, source: impl Into<String>) -> Self {
        Self::Internal {
            code,
            message: None,
            source: Some(source.into()),
        }
    }
}
