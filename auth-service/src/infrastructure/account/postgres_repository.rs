use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error, FromRow, PgPool};
use uuid::Uuid;

use crate::domain::account::entity::Account;
use crate::domain::account::repository::AccountRepository;

#[derive(Debug, FromRow)]
struct AccountRow {
    id: i64,
    public_id: String,
    username: String,
    password_hash: String,
    status: String,
    failed_login_attempts: i32,
    locked_until: Option<String>,
    last_login_at: Option<String>,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

impl From<AccountRow> for Account {
    fn from(r: AccountRow) -> Self {
        Self {
            id: r.id as u64,
            public_id: r.public_id,
            username: r.username,
            password_hash: r.password_hash,
            status: r.status,
            failed_login_attempts: r.failed_login_attempts,
            locked_until: r.locked_until,
            last_login_at: r.last_login_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
        }
    }
}

pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn list(&self) -> Result<Vec<Account>, Error> {
        let rows = sqlx::query_as::<_, AccountRow>(
            r#"SELECT id, public_id, username, password_hash, status, failed_login_attempts,
                      locked_until, last_login_at, created_at, updated_at, deleted_at
               FROM account_models ORDER BY id"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, account_id: u64) -> Result<Option<Account>, Error> {
        let row = sqlx::query_as::<_, AccountRow>(
            r#"SELECT id, public_id, username, password_hash, status, failed_login_attempts,
                      locked_until, last_login_at, created_at, updated_at, deleted_at
               FROM account_models WHERE id = $1"#,
        )
        .bind(account_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, Error> {
        let row = sqlx::query_as::<_, AccountRow>(
            r#"SELECT id, public_id, username, password_hash, status, failed_login_attempts,
                      locked_until, last_login_at, created_at, updated_at, deleted_at
               FROM account_models WHERE username = $1"#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn create(
        &self,
        username: String,
        password_hash: String,
        status: String,
        failed_login_attempts: i32,
        locked_until: Option<String>,
        last_login_at: Option<String>,
    ) -> Result<Account, Error> {
        let now = Self::now_timestamp();
        let public_id = Uuid::new_v4().to_string();
        let row = sqlx::query_as::<_, AccountRow>(
            r#"INSERT INTO account_models (
                    public_id, username, password_hash, status, failed_login_attempts,
                    locked_until, last_login_at, created_at, updated_at, deleted_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULL)
                RETURNING id, public_id, username, password_hash, status, failed_login_attempts,
                          locked_until, last_login_at, created_at, updated_at, deleted_at"#,
        )
        .bind(&public_id)
        .bind(&username)
        .bind(&password_hash)
        .bind(&status)
        .bind(failed_login_attempts)
        .bind(&locked_until)
        .bind(&last_login_at)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
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
    ) -> Result<Option<Account>, Error> {
        let current = self.find_by_id(account_id).await?;
        let Some(mut acc) = current else {
            return Ok(None);
        };
        if let Some(v) = username {
            acc.username = v;
        }
        if let Some(v) = password_hash {
            acc.password_hash = v;
        }
        if let Some(v) = status {
            acc.status = v;
        }
        if let Some(v) = failed_login_attempts {
            acc.failed_login_attempts = v;
        }
        if let Some(v) = locked_until {
            acc.locked_until = Some(v);
        }
        if let Some(v) = last_login_at {
            acc.last_login_at = Some(v);
        }
        if let Some(v) = deleted_at {
            acc.deleted_at = Some(v);
        }
        acc.updated_at = Self::now_timestamp();

        let row = sqlx::query_as::<_, AccountRow>(
            r#"UPDATE account_models SET
                    username = $2,
                    password_hash = $3,
                    status = $4,
                    failed_login_attempts = $5,
                    locked_until = $6,
                    last_login_at = $7,
                    created_at = $8,
                    updated_at = $9,
                    deleted_at = $10
                WHERE id = $1
                RETURNING id, public_id, username, password_hash, status, failed_login_attempts,
                          locked_until, last_login_at, created_at, updated_at, deleted_at"#,
        )
        .bind(account_id as i64)
        .bind(&acc.username)
        .bind(&acc.password_hash)
        .bind(&acc.status)
        .bind(acc.failed_login_attempts)
        .bind(&acc.locked_until)
        .bind(&acc.last_login_at)
        .bind(&acc.created_at)
        .bind(&acc.updated_at)
        .bind(&acc.deleted_at)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, account_id: u64) -> Result<bool, Error> {
        let result = sqlx::query(r#"DELETE FROM account_models WHERE id = $1"#)
            .bind(account_id as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
