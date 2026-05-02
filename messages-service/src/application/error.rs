#[derive(Debug)]
pub enum AppError {
    Validation {
        code: &'static str,
        message: Option<String>,
    },
    NotFound {
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

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::NotFound {
            code,
            message: Some(message.into()),
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
