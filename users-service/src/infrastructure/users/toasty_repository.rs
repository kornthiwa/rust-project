use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jiff::Timestamp;
use tokio::sync::Mutex;

use crate::domain::users::entity::{User as DomainUser, UserNameInput};
use crate::domain::users::error::DomainError;
use crate::domain::users::repository::UserRepository;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: i32,
    pub active: bool,
    #[unique]
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub deleted_at: Option<Timestamp>,
}

pub struct ToastyUserRepository {
    db: Arc<Mutex<toasty::Db>>,
}

impl ToastyUserRepository {
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

impl TryFrom<User> for DomainUser {
    type Error = DomainError;

    fn try_from(row: User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            active: row.active,
            username: row.username,
            password: row.password,
            first_name: row.first_name,
            last_name: row.last_name,
            created_at: ts_to_utc(row.created_at)?,
            updated_at: ts_to_utc(row.updated_at)?,
        })
    }
}

#[async_trait]
impl UserRepository for ToastyUserRepository {
    async fn list_users(&self) -> Result<Vec<DomainUser>, DomainError> {
        let mut db = self.db.lock().await;
        let rows = User::all()
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        rows.into_iter()
            .filter(|row| row.active && row.deleted_at.is_none())
            .map(DomainUser::try_from)
            .collect()
    }

    async fn get_user_by_id(&self, id: &str) -> Result<DomainUser, DomainError> {
        let user_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let row = User::get_by_id(&mut *db, &user_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if !row.active || row.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        DomainUser::try_from(row)
    }

    async fn create_user(&self, input: UserNameInput) -> Result<DomainUser, DomainError> {
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let created = toasty::create!(User {
            active: input.active.unwrap_or(true),
            username: input.username,
            password: input.password,
            first_name: input.first_name,
            last_name: input.last_name,
            created_at: Some(now),
            updated_at: Some(now),
            deleted_at: Option::<Timestamp>::None,
        })
        .exec(&mut *db)
        .await
        .map_err(|_| DomainError::RepositoryFailure)?;

        DomainUser::try_from(created)
    }

    async fn update_user(&self, id: &str, input: UserNameInput) -> Result<DomainUser, DomainError> {
        let user_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let mut user = User::get_by_id(&mut *db, &user_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if user.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        let next_active = input.active.unwrap_or(user.active);

        user.update()
            .active(next_active)
            .username(input.username)
            .password(input.password)
            .first_name(input.first_name)
            .last_name(input.last_name)
            .updated_at(Some(now))
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        let refreshed = User::get_by_id(&mut *db, &user_id)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        DomainUser::try_from(refreshed)
    }

    async fn delete_user(&self, id: &str) -> Result<DomainUser, DomainError> {
        let user_id = id.parse::<i32>().map_err(|_| DomainError::RepositoryFailure)?;
        let mut db = self.db.lock().await;
        let now = Timestamp::now();
        let mut user = User::get_by_id(&mut *db, &user_id)
            .await
            .map_err(|_| DomainError::NotFound)?;

        if user.deleted_at.is_some() {
            return Err(DomainError::NotFound);
        }

        user.update()
            .active(false)
            .updated_at(Some(now))
            .deleted_at(Some(now))
            .exec(&mut *db)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        let refreshed = User::get_by_id(&mut *db, &user_id)
            .await
            .map_err(|_| DomainError::RepositoryFailure)?;

        DomainUser::try_from(refreshed)
    }
}
