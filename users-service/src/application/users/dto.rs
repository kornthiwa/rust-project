use serde::Deserialize;

use crate::domain::users::entity::UserNameInput;

#[derive(Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize)]
pub struct UpdateUserDto {
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

impl From<CreateUserDto> for UserNameInput {
    fn from(dto: CreateUserDto) -> Self {
        Self {
            active: Some(true),
            username: dto.username,
            password: dto.password,
            first_name: dto.first_name,
            last_name: dto.last_name,
        }
    }
}

impl From<UpdateUserDto> for UserNameInput {
    fn from(dto: UpdateUserDto) -> Self {
        Self {
            active: None,
            username: dto.username,
            password: dto.password,
            first_name: dto.first_name,
            last_name: dto.last_name,
        }
    }
}