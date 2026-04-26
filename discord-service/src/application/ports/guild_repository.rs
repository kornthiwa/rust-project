use async_trait::async_trait;
use crate::domain::guild::Guild;

#[async_trait]
pub trait GuildRepository: Send + Sync {
    async fn save(&self, guild: Guild);
}