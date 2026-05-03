use std::sync::Arc;

use async_trait::async_trait;

use super::consumed_auth_events::ConsumedAuthEvent;
use super::message_events::MessageEvent;

/// Inbound from Kafka: this service subscribes to message events and optionally auth.events.
#[async_trait]
pub trait MessagingInboundHandler: Send + Sync {
    async fn on_message_event(&self, event: MessageEvent) -> Result<(), String>;
    async fn on_auth_event(&self, event: ConsumedAuthEvent) -> Result<(), String>;
}

pub type MessagingInboundHandlerRef = Arc<dyn MessagingInboundHandler>;
