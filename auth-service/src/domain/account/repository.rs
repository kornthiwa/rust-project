use async_trait::async_trait;
use sqlx::Error;

use crate::domain::account::entity::Account;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Account>, Error>;
    async fn find_by_id(&self, account_id: u64) -> Result<Option<Account>, Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, Error>;
    async fn create(
        &self,
        username: String,
        password_hash: String,
        status: String,
        failed_login_attempts: i32,
        locked_until: Option<String>,
        last_login_at: Option<String>,
    ) -> Result<Account, Error>;
    async fn update(
        &self,
        account_id: u64,
        username: Option<String>,
        password_hash: Option<String>,
        status: Option<String>,
        failed_login_attempts: Option<i32>,
        locked_until: Option<String>,
        last_login_at: Option<String>,
        deleted_at: Option<String>,
    ) -> Result<Option<Account>, Error>;
    async fn delete(&self, account_id: u64) -> Result<bool, Error>;
}
