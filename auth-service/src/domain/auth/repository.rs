use async_trait::async_trait;

use crate::domain::auth::entity::{AccountInfo, CreateAccountInput};
use crate::domain::auth::error::DomainError;

#[async_trait]
pub trait AuthRepository {
    async fn get_account_by_username(&self, username: &str) -> Result<AccountInfo, DomainError>;
    async fn create_account(&self, input: CreateAccountInput) -> Result<AccountInfo, DomainError>;
}
