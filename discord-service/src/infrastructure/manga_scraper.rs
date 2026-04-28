use crate::application::error::AppError;
use crate::application::ports::{MangaScraper, ScrapeResult};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpMangaScraper {
    client: Client,
}

impl HttpMangaScraper {
    pub fn new() -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| AppError::Infrastructure(format!("create http client failed: {e}")))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl MangaScraper for HttpMangaScraper {
    async fn scrape(&self, url: &str) -> Result<ScrapeResult, AppError> {
        if !(url.contains("sing-manga.com") || url.contains("xn--l3c0azab5a2gta.com")) {
            return Err(AppError::Validation(
                "ไม่รองรับเว็บไซต์นี้ กรุณาใช้ sing-manga.com หรือ สดใสเมะ.com".to_string(),
            ));
        }

        let response = self
            .client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Cache-Control", "no-cache")
            .send()
            .await
            .map_err(|e| AppError::Infrastructure(format!("request manga page failed: {e}")))?;
        if !response.status().is_success() {
            return Err(AppError::Infrastructure(format!(
                "request manga page failed with status {}",
                response.status()
            )));
        }
        let html = response
            .text()
            .await
            .map_err(|e| AppError::Infrastructure(format!("read manga html failed: {e}")))?;
        let document = Html::parse_document(&html);

        let title = select_title(&document)?;
        let image_url = select_image(&document)?;
        let (latest_chapter, latest_chapter_url) = select_latest_chapter(&document)?;

        Ok(ScrapeResult {
            title,
            latest_chapter,
            latest_chapter_url,
            image_url,
        })
    }
}

fn select_title(document: &Html) -> Result<String, AppError> {
    for query in ["h1.entry-title", r#"meta[property="og:title"]"#, "h1"] {
        let selector = Selector::parse(query)
            .map_err(|_| AppError::Infrastructure(format!("invalid selector: {query}")))?;
        if query.starts_with("meta") {
            if let Some(title) = document
                .select(&selector)
                .next()
                .and_then(|n| n.value().attr("content"))
                .map(str::to_string)
                .filter(|v| !v.trim().is_empty())
            {
                return Ok(title);
            }
            continue;
        }

        if let Some(title) = document
            .select(&selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .filter(|v| !v.is_empty())
        {
            return Ok(title);
        }
    }

    Err(AppError::NotFound("ไม่พบชื่อการ์ตูน".to_string()))
}

fn select_image(document: &Html) -> Result<Option<String>, AppError> {
    let selector = Selector::parse("div.thumb img")
        .map_err(|_| AppError::Infrastructure("invalid image selector".to_string()))?;
    Ok(document
        .select(&selector)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(str::to_string))
}

fn select_latest_chapter(document: &Html) -> Result<(i32, String), AppError> {
    let chapter_selector = Selector::parse("div.lastend div.inepcx a")
        .map_err(|_| AppError::Infrastructure("invalid chapter selector".to_string()))?;
    let chapter_number_selector = Selector::parse("span.epcurlast")
        .map_err(|_| AppError::Infrastructure("invalid chapter number selector".to_string()))?;

    let chapter = document
        .select(&chapter_selector)
        .last()
        .ok_or_else(|| AppError::NotFound("ไม่พบตอนล่าสุด".to_string()))?;
    let chapter_text = chapter
        .select(&chapter_number_selector)
        .next()
        .map(|n| n.text().collect::<String>())
        .ok_or_else(|| AppError::NotFound("ไม่พบหมายเลขตอน".to_string()))?;
    let chapter_number_text = chapter_text.replace("Chapter ", "").replace("ตอนที่ ", "");
    let latest_chapter = chapter_number_text
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse::<i32>()
        .map_err(|e| AppError::Validation(format!("แปลงหมายเลขตอนไม่สำเร็จ: {e}")))?;

    let chapter_url = chapter
        .value()
        .attr("href")
        .ok_or_else(|| AppError::NotFound("ไม่พบ URL ของตอนล่าสุด".to_string()))?
        .to_string();

    Ok((latest_chapter, chapter_url))
}
