use async_trait::async_trait;
use sqlx::Error;

use crate::domain::message::entity::Message;

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(
        &self,
        conversation_id: String,
        author_subject: String,
        body: String,
    ) -> Result<Message, Error>;

    async fn find_by_id(&self, message_id: u64) -> Result<Option<Message>, Error>;

    async fn list_by_conversation(&self, conversation_id: &str) -> Result<Vec<Message>, Error>;
}
