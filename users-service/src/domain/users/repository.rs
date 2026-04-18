use async_trait::async_trait;

use crate::domain::users::entity::{User, UserNameInput};
use crate::domain::users::error::DomainError;

#[async_trait]
pub trait UserRepository {
    async fn list_users(&self) -> Result<Vec<User>, DomainError>;
    async fn get_user_by_id(&self, id: &str) -> Result<User, DomainError>;
    async fn create_user(&self, input: UserNameInput) -> Result<User, DomainError>;
    async fn update_user(&self, id: &str, input: UserNameInput) -> Result<User, DomainError>;
    async fn delete_user(&self, id: &str) -> Result<User, DomainError>;
}