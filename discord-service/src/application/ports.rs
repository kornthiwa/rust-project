use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait MessagePublisher: Send + Sync {
    async fn send_message(&self, channel_id: u64, content: &str) -> Result<()>;
}
