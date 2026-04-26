use async_trait::async_trait;
use crate::domain::manga::Manga;

#[async_trait]
pub trait MangaRepository: Send + Sync {
    async fn save(&self, manga: Manga);
}