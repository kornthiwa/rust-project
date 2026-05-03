use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::MessageEvent;

    #[test]
    fn message_event_json_roundtrip_message_created() {
        let event = MessageEvent::MessageCreated {
            message_id: 1,
            public_id: "pub-1".into(),
            conversation_id: "conv-a".into(),
            author_subject: "user:1".into(),
            occurred_at: "2026-05-03T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: MessageEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    #[test]
    fn message_event_malformed_json_errors() {
        assert!(serde_json::from_str::<MessageEvent>("not json").is_err());
    }
}

