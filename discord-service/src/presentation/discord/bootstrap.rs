use crate::application::error::AppError;
use crate::application::ports::UpdateNotifier;
use crate::application::services::{AppServices, AutoUpdateService, ChannelService, MangaService};
use crate::domain::manga_tracking::MangaTracking;
use crate::infrastructure::manga_scraper::HttpMangaScraper;
use crate::infrastructure::mongo::MongoPool;
use crate::infrastructure::repositories::{MongoChannelRepository, MongoMangaRepository};
use crate::presentation::discord::client;
use crate::presentation::discord::handlers::DiscordEventHandler;
use async_trait::async_trait;
use std::sync::Arc;

pub fn build_application(pool: MongoPool) -> Result<Arc<AppServices>, AppError> {
    let pool = Arc::new(pool);
    let interval_seconds = std::env::var("MANGA_UPDATE_INTERVAL_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(4 * 60 * 60);
    let max_concurrency = std::env::var("MANGA_UPDATE_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(4);
    let manga_repository = Arc::new(MongoMangaRepository::new(pool.clone()));
    let channel_repository = Arc::new(MongoChannelRepository::new(pool.clone()));
    let scraper = Arc::new(HttpMangaScraper::new()?);
    let noop_notifier = Arc::new(NoopNotifier);

    Ok(Arc::new(AppServices {
        manga_service: Arc::new(MangaService::new(manga_repository.clone())),
        channel_service: Arc::new(ChannelService::new(channel_repository.clone())),
        channel_repository,
        auto_update_service: Arc::new(AutoUpdateService::new(
            manga_repository,
            scraper,
            noop_notifier,
            interval_seconds,
            max_concurrency,
        )),
    }))
}

pub async fn run_discord_bot(
    app: Arc<AppServices>,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("DISCORD_TOKEN").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ไม่พบ DISCORD_TOKEN ใน environment",
        )
    })?;

    let handler = DiscordEventHandler::new(app);
    client::run(&token, handler).await
}

#[derive(Clone)]
struct NoopNotifier;

#[async_trait]
impl UpdateNotifier for NoopNotifier {
    async fn notify_manga_updates(&self, _mangas: &[MangaTracking]) -> Result<(), AppError> {
        Ok(())
    }
}
