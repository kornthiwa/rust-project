use async_trait::async_trait;

use crate::application::ports::{AuthEvent, AuthEventInboundHandler};

/// Default [`AuthEventInboundHandler`]: structured logs only. Safe for idempotent audit/projection reads.
/// Do not publish back to the same topic from here without deduplication.
pub struct LoggingAuthEventInboundHandler;

#[async_trait]
impl AuthEventInboundHandler for LoggingAuthEventInboundHandler {
    async fn handle(&self, event: AuthEvent) -> Result<(), String> {
        tracing::info!(?event, "auth event inbound");
        Ok(())
    }
}
