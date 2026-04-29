use async_trait::async_trait;

use crate::domain::user::entity::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn list(&self) -> toasty::Result<Vec<User>>;
    async fn find_by_id(&self, user_id: u64) -> toasty::Result<Option<User>>;
    async fn create(
        &self,
        email: String,
        display_name: String,
        is_active: bool,
    ) -> toasty::Result<User>;
    async fn update(
        &self,
        user_id: u64,
        email: Option<String>,
        display_name: Option<String>,
        is_active: Option<bool>,
    ) -> toasty::Result<Option<User>>;
    async fn delete(&self, user_id: u64) -> toasty::Result<bool>;
}
