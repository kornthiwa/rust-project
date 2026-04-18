use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jiff::Timestamp;
use tokio::sync::Mutex;

use crate::domain::auth::entity::{AccountInfo, CreateAccountInput};
use crate::domain::auth::error::DomainError;
use crate::domain::auth::repository::AuthRepository;

#[derive(Debug, toasty::Model)]
pub struct Account {
    #[key]
    #[auto]
    pub id: i32,
    pub active: bool,
    #[unique]
    pub username: String,
    pub password: String,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub deleted_at: Option<Timestamp>,
}

pub struct ToastyAuthRepository {
    db: Arc<Mutex<toasty::Db>>,
}

impl ToastyAuthRepository {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

fn ts_to_utc(ts: Option<Timestamp>) -> Result<DateTime<Utc>, DomainError> {
    let parsed = ts.unwrap_or_else(Timestamp::now).to_string();
    DateTime::parse_from_rfc3339(&parsed)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| DomainError::RepositoryFailure)
}

impl TryFrom<Account> for AccountInfo {
    type Error = DomainError;

    fn try_from(value: Account) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            active: value.active,
            username: value.username,
            password: value.password,
            created_at: ts_to_utc(value.created_at)?,
            updated_at: ts_to_utc(value.updated_at)?,
        })
    }
}

#[async_trait]
impl AuthRepository for ToastyAuthRepository {
    async fn get_account_by_username(&self, username: &str) -> Result<AccountInfo, DomainError> {
        let mut db = self.db.lock().await;
        let rows = Account::all()
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        let account = rows
            .into_iter()
            .find(|row| row.username == username && row.deleted_at.is_none())
            .ok_or(DomainError::NotFound)?;

        AccountInfo::try_from(account)
    }

    async fn create_account(&self, input: CreateAccountInput) -> Result<AccountInfo, DomainError> {
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let rows = Account::all()
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        if rows
            .iter()
            .any(|row| row.username == input.username && row.deleted_at.is_none())
        {
            return Err(DomainError::Conflict);
        }

        let created = toasty::create!(Account {
            active: true,
            username: input.username,
            password: input.password,
            created_at: Some(now),
            updated_at: Some(now),
            deleted_at: Option::<Timestamp>::None,
        })
        .exec(&mut *db)
        .await
        .map_err(|_| DomainError::RepositoryFailure)?;

        AccountInfo::try_from(created)
    }
}
