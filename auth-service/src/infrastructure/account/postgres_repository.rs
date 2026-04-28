use async_trait::async_trait;
use toasty::Db;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::account::entity::Account;
use crate::domain::account::repository::AccountRepository;
use crate::infrastructure::account::account_model::AccountModel;

pub struct PostgresAccountRepository {
    db: Db,
}

impl PostgresAccountRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn list(&self) -> toasty::Result<Vec<Account>> {
        let mut db = self.db.clone();
        let models = AccountModel::all().exec(&mut db).await?;
        Ok(models.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_id(&self, account_id: u64) -> toasty::Result<Option<Account>> {
        let mut db = self.db.clone();
        let model = AccountModel::filter_by_id(account_id)
            .first()
            .exec(&mut db)
            .await?;
        Ok(model.map(Into::into))
    }

    async fn find_by_username(&self, username: &str) -> toasty::Result<Option<Account>> {
        let mut db = self.db.clone();
        let model = AccountModel::filter_by_username(username)
            .first()
            .exec(&mut db)
            .await?;
        Ok(model.map(Into::into))
    }

    async fn create(
        &self,
        username: String,
        password_hash: String,
        status: String,
        failed_login_attempts: i32,
        locked_until: Option<String>,
        last_login_at: Option<String>,
    ) -> toasty::Result<Account> {
        let mut db = self.db.clone();
        let now = Self::now_timestamp();

        let model = toasty::create!(AccountModel {
            public_id: Uuid::new_v4().to_string(),
            username,
            password_hash,
            status,
            failed_login_attempts,
            locked_until,
            last_login_at,
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
        })
        .exec(&mut db)
        .await?;

        Ok(model.into())
    }

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
    ) -> toasty::Result<Option<Account>> {
        let mut db = self.db.clone();
        let model = AccountModel::filter_by_id(account_id)
            .first()
            .exec(&mut db)
            .await?;

        let Some(mut model) = model else {
            return Ok(None);
        };

        let mut update = model.update();
        if let Some(value) = username {
            update = update.username(value);
        }
        if let Some(value) = password_hash {
            update = update.password_hash(value);
        }
        if let Some(value) = status {
            update = update.status(value);
        }
        if let Some(value) = failed_login_attempts {
            update = update.failed_login_attempts(value);
        }
        if let Some(value) = locked_until {
            update = update.locked_until(value);
        }
        if let Some(value) = last_login_at {
            update = update.last_login_at(value);
        }
        if let Some(value) = deleted_at {
            update = update.deleted_at(value);
        }
        update = update.updated_at(Self::now_timestamp());
        update.exec(&mut db).await?;

        let updated = AccountModel::filter_by_id(account_id)
            .first()
            .exec(&mut db)
            .await?;
        Ok(updated.map(Into::into))
    }

    async fn delete(&self, account_id: u64) -> toasty::Result<bool> {
        let mut db = self.db.clone();
        let model = AccountModel::filter_by_id(account_id)
            .first()
            .exec(&mut db)
            .await?;

        let Some(model) = model else {
            return Ok(false);
        };

        model.delete().exec(&mut db).await?;
        Ok(true)
    }
}
