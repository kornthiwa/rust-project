mod app;
mod config;
mod domain;
mod application;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let bot: infrastructure::discord::client::DiscordBot = app::App::build().await;

    if let Err(e) = bot.start().await {
        eprintln!("bot crashed: {e}");
    }
}