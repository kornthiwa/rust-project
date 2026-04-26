#[derive(Debug, Clone)]
pub struct Guild {
    pub guild_id: String,
    pub channel_id: String,
}

impl Guild {
    pub fn new(guild_id: String, channel_id: String) -> Self {
        Self { guild_id, channel_id }
    }
}