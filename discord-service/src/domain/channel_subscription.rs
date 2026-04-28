use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct ChannelSubscription {
    pub channel_id: String,
    pub guild_id: String,
    pub guild_name: String,
    pub channel_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChannelSubscription {
    pub fn new(
        channel_id: String,
        guild_id: String,
        guild_name: String,
        channel_name: String,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            channel_id,
            guild_id,
            guild_name,
            channel_name,
            created_at: now,
            updated_at: now,
        }
    }
}
