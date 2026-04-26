use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    application::ports::manga_repository::MangaRepository,
    domain::manga::Manga,
};

#[derive(Clone, Default)]
pub struct InMemoryMangaRepo {
    pub data: Arc<Mutex<Vec<Manga>>>,
}

#[async_trait::async_trait]
impl MangaRepository for InMemoryMangaRepo {
    async fn save(&self, manga: Manga) {
        self.data.lock().await.push(manga);
    }
}