use crate::{
    application::ports::manga_repository::MangaRepository,
    domain::manga::{Manga, MangaSite},
};

pub struct AddMangaUseCase<R: MangaRepository> {
    repo: R,
}

impl<R: MangaRepository> AddMangaUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, site: &str, title: String, url: String) -> String {
        let manga = Manga {
            site: MangaSite::from_str(site),
            title,
            url,
        };

        self.repo.save(manga).await;

        "📚 manga added successfully".to_string()
    }
}