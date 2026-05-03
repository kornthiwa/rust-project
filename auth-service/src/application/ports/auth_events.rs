use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum AuthEvent {
    #[serde(rename = "user_registered")]
    UserRegistered {
        account_id: u64,
        username: String,
        occurred_at: String,
    },
    #[serde(rename = "user_logged_in")]
    UserLoggedIn {
        account_id: u64,
        username: String,
        occurred_at: String,
        expires_in_seconds: i64,
    },
}

impl AuthEvent {
    /// Kafka message key — keep routing rules next to the event type, not in the Kafka adapter.
    pub fn partition_key(&self) -> String {
        match self {
            AuthEvent::UserRegistered { account_id, .. }
            | AuthEvent::UserLoggedIn { account_id, .. } => account_id.to_string(),
        }
    }

    pub fn user_registered(account_id: u64, username: String) -> Self {
        Self::UserRegistered {
            account_id,
            username,
            occurred_at: Utc::now().to_rfc3339(),
        }
    }
    pub fn user_logged_in(
        account_id: u64,
        username: String,
        expires_in_seconds: i64,
    ) -> Self {
        Self::UserLoggedIn {
            account_id,
            username,
            occurred_at: Utc::now().to_rfc3339(),
            expires_in_seconds,
        }
    }
}

#[async_trait]
pub trait AuthEventPublisher: Send + Sync {
    async fn publish(&self, event: AuthEvent) -> Result<(), String>;
}

/// Inbound counterpart to [`AuthEventPublisher`]: run after an [`AuthEvent`] is read from the bus.
/// Add variants on [`AuthEvent`] for new payloads; add types implementing this trait (or compose them) for new reactions.
/// Implementations must be idempotent where the same event may be redelivered.
#[async_trait]
pub trait AuthEventInboundHandler: Send + Sync {
    async fn handle(&self, event: AuthEvent) -> Result<(), String>;
}

/// Use in [`crate::app::Bootstrap`] and Kafka wiring so the consumer stays transport-agnostic.
pub type AuthEventInboundHandlerRef = Arc<dyn AuthEventInboundHandler>;

#[cfg(test)]
mod tests {
    use super::AuthEvent;

    #[test]
    fn auth_event_json_roundtrip_user_registered() {
        let event = AuthEvent::UserRegistered {
            account_id: 42,
            username: "alice".into(),
            occurred_at: "2026-05-03T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: AuthEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    #[test]
    fn auth_event_json_roundtrip_user_logged_in() {
        let event = AuthEvent::UserLoggedIn {
            account_id: 7,
            username: "bob".into(),
            occurred_at: "2026-05-03T12:00:00Z".into(),
            expires_in_seconds: 3600,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let back: AuthEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, back);
    }

    #[test]
    fn auth_event_malformed_json_errors() {
        assert!(serde_json::from_str::<AuthEvent>("not json").is_err());
    }

    #[test]
    fn auth_event_unknown_type_errors() {
        let raw = r#"{"type":"unknown_event","data":{}}"#;
        assert!(serde_json::from_str::<AuthEvent>(raw).is_err());
    }
}
