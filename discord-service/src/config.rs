use anyhow::{Context, Result};

pub struct AppConfig {
    pub discord_token: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let discord_token =
            std::env::var("DISCORD_TOKEN").context("missing DISCORD_TOKEN environment variable")?;

        Ok(Self { discord_token })
    }
}
