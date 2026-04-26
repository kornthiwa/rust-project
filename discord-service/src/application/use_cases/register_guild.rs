use crate::{
    application::ports::guild_repository::GuildRepository,
    domain::guild::Guild,
};

pub struct RegisterGuildUseCase<R: GuildRepository> {
    repo: R,
}

impl<R: GuildRepository> RegisterGuildUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, guild_id: String, channel_id: String) -> String {
        let guild = Guild::new(guild_id, channel_id);

        self.repo.save(guild).await;

        "✅ guild registered".to_string()
    }
}