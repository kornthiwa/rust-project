use std::sync::Arc;

use serde::Deserialize;

use crate::domain::account::entity::Account;
use crate::domain::account::repository::AccountRepository;

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub username: String,
    pub password_hash: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountInput {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub status: Option<String>,
    pub failed_login_attempts: Option<i32>,
    pub locked_until: Option<String>,
    pub last_login_at: Option<String>,
    pub deleted_at: Option<String>,
}

pub struct AccountService {
    repository: Arc<dyn AccountRepository>,
}

impl AccountService {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_accounts(&self) -> toasty::Result<Vec<Account>> {
        let accounts = self.repository.list().await?;
        println!("[MOCK CRUD] list accounts: {:?}", accounts);
        Ok(accounts)
    }

    pub async fn get_account(&self, account_id: u64) -> toasty::Result<Option<Account>> {
        let account = self.repository.find_by_id(account_id).await?;
        println!("[MOCK CRUD] get account {}: {:?}", account_id, account);
        Ok(account)
    }

    pub async fn create_account(&self, input: CreateAccountInput) -> toasty::Result<Account> {
        let status = input.status.unwrap_or_else(|| "active".to_string());
        let account = self
            .repository
            .create(
                input.username,
                input.password_hash,
                status,
                0,
                None,
                None,
            )
            .await?;
        println!("[MOCK CRUD] create account: {:?}", account);
        Ok(account)
    }

    pub async fn update_account(
        &self,
        account_id: u64,
        input: UpdateAccountInput,
    ) -> toasty::Result<Option<Account>> {
        let account = self
            .repository
            .update(
                account_id,
                input.username,
                input.password_hash,
                input.status,
                input.failed_login_attempts,
                input.locked_until,
                input.last_login_at,
                input.deleted_at,
            );
        let account = account.await?;
        println!("[MOCK CRUD] update account {}: {:?}", account_id, account);
        Ok(account)
    }

    pub async fn delete_account(&self, account_id: u64) -> toasty::Result<bool> {
        let is_deleted = self.repository.delete(account_id).await?;
        println!("[MOCK CRUD] delete account {}: {}", account_id, is_deleted);
        Ok(is_deleted)
    }
}
