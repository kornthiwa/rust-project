use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageEvent {
    #[serde(rename = "message_created")]
    MessageCreated {
        message_id: u64,
        public_id: String,
        conversation_id: String,
        author_subject: String,
        occurred_at: String,
    },
}

impl MessageEvent {
    /// Kafka message key — route by conversation so room messages stay ordered per partition.
    pub fn partition_key(&self) -> String {
        match self {
            MessageEvent::MessageCreated {
                conversation_id, ..
            } => conversation_id.clone(),
        }
    }

    pub fn message_created(
        message_id: u64,
        public_id: String,
        conversation_id: String,
        author_subject: String,
    ) -> Self {
        Self::MessageCreated {
            message_id,
            public_id,
            conversation_id,
            author_subject,
            occurred_at: Utc::now().to_rfc3339(),
        }
    }
}

#[async_trait]
pub trait MessageEventPublisher: Send + Sync {
    async fn publish(&self, event: MessageEvent) -> Result<(), String>;
}
