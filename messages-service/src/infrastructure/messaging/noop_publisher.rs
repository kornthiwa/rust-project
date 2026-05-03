use async_trait::async_trait;

use crate::application::ports::{MessageEvent, MessageEventPublisher};

pub struct NoopMessageEventPublisher;

#[async_trait]
impl MessageEventPublisher for NoopMessageEventPublisher {
    async fn publish(&self, _event: MessageEvent) -> Result<(), String> {
        Ok(())
    }
}
