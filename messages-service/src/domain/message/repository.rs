use async_trait::async_trait;

use crate::domain::message::entity::Message;

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(
        &self,
        conversation_id: String,
        author_subject: String,
        body: String,
    ) -> toasty::Result<Message>;

    async fn find_by_id(&self, message_id: u64) -> toasty::Result<Option<Message>>;

    async fn list_by_conversation(&self, conversation_id: &str) -> toasty::Result<Vec<Message>>;
}
