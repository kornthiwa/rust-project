use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error, FromRow, PgPool};
use uuid::Uuid;

use crate::domain::user::entity::User;
use crate::domain::user::repository::UserRepository;

#[derive(Debug, FromRow)]
struct UserRow {
    id: i64,
    public_id: String,
    email: String,
    display_name: String,
    is_active: bool,
    created_at: String,
    updated_at: String,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        Self {
            id: r.id as u64,
            public_id: r.public_id,
            email: r.email,
            display_name: r.display_name,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn list(&self) -> Result<Vec<User>, Error> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, public_id, email, display_name, is_active, created_at, updated_at
               FROM user_models ORDER BY id"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, user_id: u64) -> Result<Option<User>, Error> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, public_id, email, display_name, is_active, created_at, updated_at
               FROM user_models WHERE id = $1"#,
        )
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn create(
        &self,
        email: String,
        display_name: String,
        is_active: bool,
    ) -> Result<User, Error> {
        let now = Self::now_timestamp();
        let public_id = Uuid::new_v4().to_string();
        let row = sqlx::query_as::<_, UserRow>(
            r#"INSERT INTO user_models (public_id, email, display_name, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, public_id, email, display_name, is_active, created_at, updated_at"#,
        )
        .bind(&public_id)
        .bind(&email)
        .bind(&display_name)
        .bind(is_active)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn update(
        &self,
        user_id: u64,
        email: Option<String>,
        display_name: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Option<User>, Error> {
        let current = self.find_by_id(user_id).await?;
        let Some(mut user) = current else {
            return Ok(None);
        };
        if let Some(v) = email {
            user.email = v;
        }
        if let Some(v) = display_name {
            user.display_name = v;
        }
        if let Some(v) = is_active {
            user.is_active = v;
        }
        user.updated_at = Self::now_timestamp();

        let row = sqlx::query_as::<_, UserRow>(
            r#"UPDATE user_models SET
                    email = $2,
                    display_name = $3,
                    is_active = $4,
                    created_at = $5,
                    updated_at = $6
                WHERE id = $1
                RETURNING id, public_id, email, display_name, is_active, created_at, updated_at"#,
        )
        .bind(user_id as i64)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(user.is_active)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, user_id: u64) -> Result<bool, Error> {
        let result = sqlx::query(r#"DELETE FROM user_models WHERE id = $1"#)
            .bind(user_id as i64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
