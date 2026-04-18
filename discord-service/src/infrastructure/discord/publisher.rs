use anyhow::{Context, Result};
use async_trait::async_trait;
use serenity::{http::Http, model::id::ChannelId};
use std::sync::Arc;

use crate::application::ports::MessagePublisher;

pub struct SerenityMessagePublisher {
    http: Arc<Http>,
}

impl SerenityMessagePublisher {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl MessagePublisher for SerenityMessagePublisher {
    async fn send_message(&self, channel_id: u64, content: &str) -> Result<()> {
        ChannelId::new(channel_id)
            .say(&self.http, content)
            .await
            .context("failed to send Discord message")?;

        Ok(())
    }
}
