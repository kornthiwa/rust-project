use async_trait::async_trait;

use crate::application::ports::{AuthEvent, AuthEventPublisher};

pub struct NoopAuthEventPublisher;

#[async_trait]
impl AuthEventPublisher for NoopAuthEventPublisher {
    async fn publish(&self, _event: AuthEvent) -> Result<(), String> {
        Ok(())
    }
}
