use crate::application::error::AppError;
use crate::application::ports::{ChannelRepository, UpdateNotifier};
use crate::domain::manga_tracking::MangaTracking;
use async_trait::async_trait;
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, CreateEmbedFooter, CreateMessage};
use std::sync::Arc;

#[derive(Clone)]
pub struct DiscordNotifier {
    channel_repository: Arc<dyn ChannelRepository>,
    context: Context,
}

impl DiscordNotifier {
    pub fn new(channel_repository: Arc<dyn ChannelRepository>, context: Context) -> Self {
        Self {
            channel_repository,
            context,
        }
    }
}

#[async_trait]
impl UpdateNotifier for DiscordNotifier {
    async fn notify_manga_updates(&self, mangas: &[MangaTracking]) -> Result<(), AppError> {
        let channels = self.channel_repository.list_all().await?;
        let channel_ids = channels
            .into_iter()
            .filter_map(|c| c.channel_id.parse::<u64>().ok())
            .map(ChannelId::new)
            .collect::<Vec<_>>();

        if channel_ids.is_empty() {
            return Ok(());
        }

        for manga in mangas {
            let embed = CreateEmbed::new()
                .title(format!("การอัพเดทมังงะ: {}", manga.title))
                .description(format!("อัพเดทถึงตอนที่ {}", manga.latest_chapter))
                .field("ชื่อมังงะ", &manga.title, true)
                .field("ตอนล่าสุด", format!("ตอนที่ {}", manga.latest_chapter), true)
                .field("ลิงก์ตอนล่าสุด", &manga.latest_chapter_url, false)
                .color(Colour::DARK_GREEN)
                .footer(CreateEmbedFooter::new("ระบบอัพเดทมังงะอัตโนมัติ"));
            let embed = match &manga.image_url {
                Some(image_url) => embed.thumbnail(image_url),
                None => embed,
            };

            for channel_id in &channel_ids {
                let message = CreateMessage::new().embed(embed.clone());
                channel_id
                    .send_message(&self.context.http, message)
                    .await
                    .map_err(|e| AppError::Infrastructure(format!("send discord message failed: {e}")))?;
            }
        }

        Ok(())
    }
}
