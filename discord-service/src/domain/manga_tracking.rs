use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct MangaTracking {
    pub title: String,
    pub url: String,
    pub latest_chapter: i32,
    pub latest_chapter_url: String,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MangaTracking {
    pub fn new_placeholder(url: String, now: DateTime<Utc>) -> Self {
        Self {
            title: "Untitled".to_string(),
            latest_chapter: 0,
            latest_chapter_url: url.clone(),
            image_url: None,
            created_at: now,
            updated_at: now,
            url,
        }
    }
}
