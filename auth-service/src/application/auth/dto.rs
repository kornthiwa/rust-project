use serde::Deserialize;

use crate::domain::auth::entity::{CreateAccountInput, LoginInput};

#[derive(Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateAccountDto {
    pub username: String,
    pub password: String,
}

impl From<LoginDto> for LoginInput {
    fn from(value: LoginDto) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}

impl From<CreateAccountDto> for CreateAccountInput {
    fn from(value: CreateAccountDto) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
