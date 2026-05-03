use async_trait::async_trait;

use crate::application::ports::{
    ConsumedAuthEvent, MessageEvent, MessagingInboundHandler,
};

/// Default inbound: structured logs only. Extend with idempotent side effects as needed.
pub struct LoggingMessagingInboundHandler;

#[async_trait]
impl MessagingInboundHandler for LoggingMessagingInboundHandler {
    async fn on_message_event(&self, event: MessageEvent) -> Result<(), String> {
        tracing::info!(?event, "message event inbound");
        Ok(())
    }

    async fn on_auth_event(&self, event: ConsumedAuthEvent) -> Result<(), String> {
        tracing::info!(?event, "auth feed inbound");
        Ok(())
    }
}
