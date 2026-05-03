use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
