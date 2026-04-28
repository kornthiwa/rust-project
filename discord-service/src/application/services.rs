use crate::application::error::AppError;
use crate::application::ports::{ChannelRepository, MangaRepository, MangaScraper, UpdateNotifier};
use crate::domain::channel_subscription::ChannelSubscription;
use crate::domain::manga_tracking::MangaTracking;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::time::{self, Duration};

#[derive(Clone)]
pub struct AppServices {
    pub manga_service: Arc<MangaService>,
    pub channel_service: Arc<ChannelService>,
    pub channel_repository: Arc<dyn ChannelRepository>,
    pub auto_update_service: Arc<AutoUpdateService>,
}

#[derive(Clone)]
pub struct MangaService {
    repository: Arc<dyn MangaRepository>,
}

impl MangaService {
    pub fn new(repository: Arc<dyn MangaRepository>) -> Self {
        Self { repository }
    }

    pub async fn add_from_url(&self, raw_url: &str) -> Result<AddMangaResult, AppError> {
        if raw_url.trim().is_empty() {
            return Err(AppError::Validation("กรุณาระบุ URL ของการ์ตูน".to_string()));
        }
        if !raw_url.starts_with("https://") {
            return Err(AppError::Validation("URL ต้องขึ้นต้นด้วย https://".to_string()));
        }

        let normalized_url = raw_url.replace("สดใสเมะ.com", "xn--l3c0azab5a2gta.com");
        if self.repository.find_by_url(&normalized_url).await?.is_some() {
            return Ok(AddMangaResult::AlreadyExists);
        }

        let manga = MangaTracking::new_placeholder(normalized_url, Utc::now());
        self.repository.create(&manga).await?;
        Ok(AddMangaResult::Created(manga))
    }
}

#[derive(Clone)]
pub enum AddMangaResult {
    AlreadyExists,
    Created(MangaTracking),
}

#[derive(Clone)]
pub struct ChannelService {
    repository: Arc<dyn ChannelRepository>,
}

impl ChannelService {
    pub fn new(repository: Arc<dyn ChannelRepository>) -> Self {
        Self { repository }
    }

    pub async fn register_channel(
        &self,
        guild_id: String,
        guild_name: String,
        channel_id: String,
        channel_name: String,
    ) -> Result<(), AppError> {
        let channel = ChannelSubscription::new(
            channel_id,
            guild_id,
            guild_name,
            channel_name,
            Utc::now(),
        );
        self.repository.upsert_for_guild(&channel).await
    }

    pub async fn list_channels(&self, guild_id: &str) -> Result<Vec<ChannelSubscription>, AppError> {
        self.repository.list_by_guild(guild_id).await
    }
}

#[derive(Clone)]
pub struct AutoUpdateService {
    manga_repository: Arc<dyn MangaRepository>,
    scraper: Arc<dyn MangaScraper>,
    notifier: Arc<dyn UpdateNotifier>,
    interval_seconds: u64,
    max_concurrency: usize,
}

impl AutoUpdateService {
    pub fn new(
        manga_repository: Arc<dyn MangaRepository>,
        scraper: Arc<dyn MangaScraper>,
        notifier: Arc<dyn UpdateNotifier>,
        interval_seconds: u64,
        max_concurrency: usize,
    ) -> Self {
        Self {
            manga_repository,
            scraper,
            notifier,
            interval_seconds,
            max_concurrency: max_concurrency.max(1),
        }
    }
    pub fn clone_with_notifier(&self, notifier: Arc<dyn UpdateNotifier>) -> Self {
        Self {
            manga_repository: self.manga_repository.clone(),
            scraper: self.scraper.clone(),
            notifier,
            interval_seconds: self.interval_seconds,
            max_concurrency: self.max_concurrency,
        }
    }

    pub async fn run_periodic_update(&self) {
        let mut interval = time::interval(Duration::from_secs(self.interval_seconds));
        loop {
            interval.tick().await;
            println!("บอทกำลังทำการอัปเดตการ์ตูนอัตโนมัติ");
            if let Err(error) = self.run_once().await {
                eprintln!("auto update failed: {error}");
            }
        }
    }

    pub async fn run_once(&self) -> Result<(), AppError> {
        let mangas = self.manga_repository.list_all().await?;
        if mangas.is_empty() {
            return Ok(());
        }

        let tasks = stream::iter(mangas.into_iter().map(|manga| {
            let manga_repository = self.manga_repository.clone();
            let scraper = self.scraper.clone();
            let notifier = self.notifier.clone();

            async move {
                let manga_url = manga.url.clone();
                let scrape = match scraper.scrape(&manga.url).await {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("scrape failed for {}: {error}", manga.url);
                        return Ok::<(), AppError>(());
                    }
                };

                if scrape.latest_chapter <= manga.latest_chapter {
                    return Ok(());
                }

                let updated = MangaTracking {
                    title: scrape.title,
                    url: manga_url.clone(),
                    latest_chapter: scrape.latest_chapter,
                    latest_chapter_url: scrape.latest_chapter_url,
                    image_url: scrape.image_url,
                    created_at: manga.created_at,
                    updated_at: Utc::now(),
                };

                manga_repository.update_latest(&updated).await?;
                if let Err(error) = notifier.notify_manga_updates(&[updated]).await {
                    eprintln!("notify update failed for {}: {error}", manga_url);
                }
                Ok(())
            }
        }))
        .buffer_unordered(self.max_concurrency);

        let results = tasks.collect::<Vec<Result<(), AppError>>>().await;
        for result in results {
            result?;
        }
        Ok(())
    }
}
