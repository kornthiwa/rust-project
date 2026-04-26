use anyhow::{Result, Context};
use serenity::{Client, all::GatewayIntents};

use crate::{config::AppConfig, presentation::discord::handler::DiscordHandler};

pub struct DiscordBot {
    client: Client,
}

impl DiscordBot {
    pub async fn new(config: AppConfig, handler: DiscordHandler) -> Result<Self> {
        let intents = GatewayIntents::GUILDS;

        let client = Client::builder(config.discord_token, intents)
            .event_handler(handler)
            .await
            .context("failed to create discord client")?;

        Ok(Self { client })
    }

    pub async fn start(mut self) -> Result<()> {
        self.client.start().await?;
        Ok(())
    }
}