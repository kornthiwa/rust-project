mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use anyhow::Result;
use application::use_cases::handle_message::HandleSlashCommandUseCase;
use config::AppConfig;
use presentation::discord::bot::DiscordBot;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;
    let use_case = HandleSlashCommandUseCase::new();
    let mut bot = DiscordBot::new(config, use_case).await?;

    bot.start().await
}
