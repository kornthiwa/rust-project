use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum UserEvent {
    #[serde(rename = "user_created")]
    UserCreated {
        user_id: u64,
        public_id: String,
        email: String,
        display_name: String,
        occurred_at: String,
    },
    #[serde(rename = "user_updated")]
    UserUpdated {
        user_id: u64,
        public_id: String,
        email: String,
        display_name: String,
        occurred_at: String,
    },
}

impl UserEvent {
    pub fn partition_key(&self) -> String {
        match self {
            UserEvent::UserCreated { user_id, .. } | UserEvent::UserUpdated { user_id, .. } => {
                user_id.to_string()
            }
        }
    }

    pub fn user_created(
        user_id: u64,
        public_id: String,
        email: String,
        display_name: String,
    ) -> Self {
        Self::UserCreated {
            user_id,
            public_id,
            email,
            display_name,
            occurred_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn user_updated(
        user_id: u64,
        public_id: String,
        email: String,
        display_name: String,
    ) -> Self {
        Self::UserUpdated {
            user_id,
            public_id,
            email,
            display_name,
            occurred_at: Utc::now().to_rfc3339(),
        }
    }
}

#[async_trait]
pub trait UserEventPublisher: Send + Sync {
    async fn publish(&self, event: UserEvent) -> Result<(), String>;
}

/// Inbound after a [`UserEvent`] is read from the bus (projections, audit).
#[async_trait]
pub trait UserEventInboundHandler: Send + Sync {
    async fn handle(&self, event: UserEvent) -> Result<(), String>;
}

pub type UserEventInboundHandlerRef = Arc<dyn UserEventInboundHandler>;

#[cfg(test)]
mod tests {
    use super::UserEvent;

    #[test]
    fn user_event_json_roundtrip_created() {
        let event = UserEvent::UserCreated {
            user_id: 1,
            public_id: "u_1".into(),
            email: "a@b.c".into(),
            display_name: "A".into(),
            occurred_at: "2026-05-03T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: UserEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    #[test]
    fn user_event_malformed_json_errors() {
        assert!(serde_json::from_str::<UserEvent>("not json").is_err());
    }
}
