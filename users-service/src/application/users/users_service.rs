use std::sync::Arc;

use crate::application::users::dto::{CreateUserDto, UpdateUserDto};
use crate::application::users::error::AppError;
use crate::domain::users::entity::User;
use crate::domain::users::repository::UserRepository;

type Result<T> = std::result::Result<T, AppError>;
type Repository = Arc<dyn UserRepository + Send + Sync>;

pub struct UserService {
    repository: Repository,
}

impl UserService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        self.repository.list_users().await.map_err(AppError::from)
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        self.repository.get_user_by_id(id).await.map_err(AppError::from)
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> Result<User> {
        self.repository
            .create_user(dto.into())
            .await
            .map_err(AppError::from)
    }

    pub async fn update_user(&self, id: &str, dto: UpdateUserDto) -> Result<User> {
        self.repository
            .update_user(id, dto.into())
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_user(&self, id: &str) -> Result<User> {
        self.repository.delete_user(id).await.map_err(AppError::from)
    }
}