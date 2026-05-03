use async_trait::async_trait;

use crate::application::ports::{UserEvent, UserEventPublisher};

pub struct NoopUserEventPublisher;

#[async_trait]
impl UserEventPublisher for NoopUserEventPublisher {
    async fn publish(&self, _event: UserEvent) -> Result<(), String> {
        Ok(())
    }
}
