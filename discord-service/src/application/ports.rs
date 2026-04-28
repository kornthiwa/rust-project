use crate::application::error::AppError;
use crate::domain::channel_subscription::ChannelSubscription;
use crate::domain::manga_tracking::MangaTracking;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct ScrapeResult {
    pub title: String,
    pub latest_chapter: i32,
    pub latest_chapter_url: String,
    pub image_url: Option<String>,
}

#[async_trait]
pub trait MangaRepository: Send + Sync {
    async fn find_by_url(&self, url: &str) -> Result<Option<MangaTracking>, AppError>;
    async fn create(&self, manga: &MangaTracking) -> Result<(), AppError>;
    async fn update_latest(&self, manga: &MangaTracking) -> Result<(), AppError>;
    async fn list_all(&self) -> Result<Vec<MangaTracking>, AppError>;
}

#[async_trait]
pub trait ChannelRepository: Send + Sync {
    async fn upsert_for_guild(&self, channel: &ChannelSubscription) -> Result<(), AppError>;
    async fn list_by_guild(&self, guild_id: &str) -> Result<Vec<ChannelSubscription>, AppError>;
    async fn list_all(&self) -> Result<Vec<ChannelSubscription>, AppError>;
}

#[async_trait]
pub trait MangaScraper: Send + Sync {
    async fn scrape(&self, url: &str) -> Result<ScrapeResult, AppError>;
}

#[async_trait]
pub trait UpdateNotifier: Send + Sync {
    async fn notify_manga_updates(&self, mangas: &[MangaTracking]) -> Result<(), AppError>;
}
