use async_trait::async_trait;

use crate::application::ports::{UserEvent, UserEventInboundHandler};

/// Default inbound: structured logs only. Safe for idempotent reads; avoid republish loops.
pub struct LoggingUserEventInboundHandler;

#[async_trait]
impl UserEventInboundHandler for LoggingUserEventInboundHandler {
    async fn handle(&self, event: UserEvent) -> Result<(), String> {
        tracing::info!(?event, "user event inbound");
        Ok(())
    }
}
