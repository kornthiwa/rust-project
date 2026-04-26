use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    application::ports::guild_repository::GuildRepository,
    domain::guild::Guild,
};

#[derive(Clone, Default)]
pub struct InMemoryGuildRepo {
    pub data: Arc<Mutex<Vec<Guild>>>,
}

#[async_trait::async_trait]
impl GuildRepository for InMemoryGuildRepo {
    async fn save(&self, guild: Guild) {
        let mut db = self.data.lock().await;
        db.push(guild);
    }
}