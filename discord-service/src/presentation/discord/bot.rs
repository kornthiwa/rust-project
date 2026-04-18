use anyhow::{Context as AnyhowContext, Result};
use serenity::{all::GatewayIntents, client::Client};

use crate::{application::use_cases::handle_message::HandleSlashCommandUseCase, config::AppConfig};

use super::handler::DiscordEventHandler;

pub struct DiscordBot {
    client: Client,
}

impl DiscordBot {
    pub async fn new(config: AppConfig, use_case: HandleSlashCommandUseCase) -> Result<Self> {
        let handler = DiscordEventHandler::new(use_case);

        let intents = GatewayIntents::GUILDS;
        let client = Client::builder(config.discord_token, intents)
            .event_handler(handler)
            .await
            .context("failed to create Discord client")?;

        Ok(Self { client })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.client
            .start()
            .await
            .context("Discord client stopped unexpectedly")
    }
}
