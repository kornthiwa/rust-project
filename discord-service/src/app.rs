use crate::{
    config::AppConfig,
    application::use_cases::{
        register_guild::RegisterGuildUseCase,
        add_manga::AddMangaUseCase,
    },
    infrastructure::memory::{
        guild_repo::InMemoryGuildRepo,
        manga_repo::InMemoryMangaRepo,
    },
    presentation::discord::handler::DiscordHandler,
    infrastructure::discord::client::DiscordBot,
};

pub struct App;

impl App {
    pub async fn build() -> DiscordBot {
        let config: AppConfig = AppConfig::from_env().unwrap();

        // repositories
        let guild_repo: InMemoryGuildRepo = InMemoryGuildRepo::default();
        let manga_repo: InMemoryMangaRepo = InMemoryMangaRepo::default();

        // use cases
        let register_uc: RegisterGuildUseCase<InMemoryGuildRepo> = RegisterGuildUseCase::new(guild_repo);
        let add_manga_uc: AddMangaUseCase<InMemoryMangaRepo> = AddMangaUseCase::new(manga_repo);

        // handler (inject both commands)
        let handler: DiscordHandler = DiscordHandler::new(register_uc, add_manga_uc);

        DiscordBot::new(config, handler)
            .await
            .expect("failed to create bot")
    }
}