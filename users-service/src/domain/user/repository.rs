use async_trait::async_trait;
use sqlx::Error;

use crate::domain::user::entity::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<User>, Error>;
    async fn find_by_id(&self, user_id: u64) -> Result<Option<User>, Error>;
    async fn create(
        &self,
        email: String,
        display_name: String,
        is_active: bool,
    ) -> Result<User, Error>;
    async fn update(
        &self,
        user_id: u64,
        email: Option<String>,
        display_name: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Option<User>, Error>;
    async fn delete(&self, user_id: u64) -> Result<bool, Error>;
}
