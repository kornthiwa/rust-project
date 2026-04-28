use async_trait::async_trait;

use crate::domain::account::entity::Account;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn list(&self) -> toasty::Result<Vec<Account>>;
    async fn find_by_id(&self, account_id: u64) -> toasty::Result<Option<Account>>;
    async fn create(
        &self,
        username: String,
        password_hash: String,
        status: String,
        failed_login_attempts: i32,
        locked_until: Option<String>,
        last_login_at: Option<String>,
    ) -> toasty::Result<Account>;
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
    ) -> toasty::Result<Option<Account>>;
    async fn delete(&self, account_id: u64) -> toasty::Result<bool>;
}
