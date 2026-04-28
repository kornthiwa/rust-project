use crate::presentation::discord::handlers::DiscordEventHandler;
use serenity::all::{Client, GatewayIntents};

pub async fn run(
    token: &str,
    handler: DiscordEventHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await?;
    client.start().await?;
    Ok(())
}
