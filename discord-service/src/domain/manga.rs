#[derive(Debug, Clone)]
pub struct Manga {
    pub site: MangaSite,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum MangaSite {
    MangaDex,
    MangaPlus,
    Custom,
}

impl MangaSite {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mangadex" => Self::MangaDex,
            "mangaplus" => Self::MangaPlus,
            _ => Self::Custom,
        }
    }
}