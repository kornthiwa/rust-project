use async_trait::async_trait;
use chrono::Utc;
use toasty::Db;
use uuid::Uuid;

use crate::domain::user::entity::User;
use crate::domain::user::repository::UserRepository;
use crate::infrastructure::user::user_model::UserModel;

pub struct PostgresUserRepository {
    db: Db,
}

impl PostgresUserRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn list(&self) -> toasty::Result<Vec<User>> {
        let mut db = self.db.clone();
        let models = UserModel::all().exec(&mut db).await?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, user_id: u64) -> toasty::Result<Option<User>> {
        let mut db = self.db.clone();
        let model = UserModel::filter_by_id(user_id)
            .first()
            .exec(&mut db)
            .await?;
        Ok(model.map(Into::into))
    }

    async fn create(
        &self,
        email: String,
        display_name: String,
        is_active: bool,
    ) -> toasty::Result<User> {
        let mut db = self.db.clone();
        let now = Self::now_timestamp();

        let model = toasty::create!(UserModel {
            public_id: Uuid::new_v4().to_string(),
            email,
            display_name,
            is_active,
            created_at: now.clone(),
            updated_at: now,
        })
        .exec(&mut db)
        .await?;

        Ok(model.into())
    }

    async fn update(
        &self,
        user_id: u64,
        email: Option<String>,
        display_name: Option<String>,
        is_active: Option<bool>,
    ) -> toasty::Result<Option<User>> {
        let mut db = self.db.clone();
        let model = UserModel::filter_by_id(user_id)
            .first()
            .exec(&mut db)
            .await?;
        let Some(mut model) = model else {
            return Ok(None);
        };

        let mut update = model.update();
        if let Some(value) = email {
            update = update.email(value);
        }
        if let Some(value) = display_name {
            update = update.display_name(value);
        }
        if let Some(value) = is_active {
            update = update.is_active(value);
        }
        update = update.updated_at(Self::now_timestamp());
        update.exec(&mut db).await?;

        let updated = UserModel::filter_by_id(user_id)
            .first()
            .exec(&mut db)
            .await?;
        Ok(updated.map(Into::into))
    }

    async fn delete(&self, user_id: u64) -> toasty::Result<bool> {
        let mut db = self.db.clone();
        let model = UserModel::filter_by_id(user_id)
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
