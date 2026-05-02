use async_trait::async_trait;
use chrono::Utc;
use toasty::Db;
use uuid::Uuid;

use crate::domain::message::entity::Message;
use crate::domain::message::repository::MessageRepository;
use crate::infrastructure::message::message_model::MessageModel;

pub struct PostgresMessageRepository {
    db: Db,
}

impl PostgresMessageRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn create(
        &self,
        conversation_id: String,
        author_subject: String,
        body: String,
    ) -> toasty::Result<Message> {
        let mut db = self.db.clone();
        let now = Self::now_timestamp();

        let model = toasty::create!(MessageModel {
            public_id: Uuid::new_v4().to_string(),
            conversation_id,
            author_subject,
            body,
            created_at: now,
        })
        .exec(&mut db)
        .await?;

        Ok(model.into())
    }

    async fn find_by_id(&self, message_id: u64) -> toasty::Result<Option<Message>> {
        let mut db = self.db.clone();
        let model = MessageModel::filter_by_id(message_id)
            .first()
            .exec(&mut db)
            .await?;
        Ok(model.map(Into::into))
    }

    async fn list_by_conversation(&self, conversation_id: &str) -> toasty::Result<Vec<Message>> {
        let mut db = self.db.clone();
        let models = MessageModel::filter(
            MessageModel::fields()
                .conversation_id()
                .eq(conversation_id.to_string()),
        )
        .exec(&mut db)
        .await?;

        let mut messages: Vec<Message> = models.into_iter().map(Into::into).collect();
        messages.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        Ok(messages)
    }
}
