use crate::application::error::AppError;
use crate::application::ports::{ChannelRepository, MangaRepository};
use crate::domain::channel_subscription::ChannelSubscription;
use crate::domain::manga_tracking::MangaTracking;
use crate::infrastructure::mongo::MongoPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::Collection;
use mongodb::bson::{DateTime as BsonDateTime, doc, oid::ObjectId};
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct MongoMangaRepository {
    pool: Arc<MongoPool>,
}

impl MongoMangaRepository {
    pub fn new(pool: Arc<MongoPool>) -> Self {
        Self { pool }
    }

    fn collection(&self) -> Collection<MangaModel> {
        self.pool.database().collection("mangas")
    }
}

#[async_trait]
impl MangaRepository for MongoMangaRepository {
    async fn find_by_url(&self, url: &str) -> Result<Option<MangaTracking>, AppError> {
        let found = self
            .collection()
            .find_one(doc! { "url": url })
            .await
            .map_err(|e| AppError::Infrastructure(format!("find manga by url failed: {e}")))?;
        Ok(found.map(|m| m.into_domain()))
    }

    async fn create(&self, manga: &MangaTracking) -> Result<(), AppError> {
        self.collection()
            .insert_one(MangaModel::from_domain(manga))
            .await
            .map_err(|e| AppError::Infrastructure(format!("create manga failed: {e}")))?;
        Ok(())
    }

    async fn update_latest(&self, manga: &MangaTracking) -> Result<(), AppError> {
        self.collection()
            .update_one(
                doc! { "url": &manga.url },
                doc! {
                    "$set": {
                        "title": &manga.title,
                        "latest_chapter": manga.latest_chapter,
                        "latest_chapter_url": &manga.latest_chapter_url,
                        "image_url": &manga.image_url,
                        "updated_at": BsonDateTime::from_system_time(manga.updated_at.into())
                    }
                },
            )
            .await
            .map_err(|e| AppError::Infrastructure(format!("update manga failed: {e}")))?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<MangaTracking>, AppError> {
        let mut cursor = self
            .collection()
            .find(doc! {})
            .await
            .map_err(|e| AppError::Infrastructure(format!("list mangas failed: {e}")))?;
        let mut result = Vec::new();
        while let Some(item) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Infrastructure(format!("iterate mangas failed: {e}")))?
        {
            result.push(item.into_domain());
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub struct MongoChannelRepository {
    pool: Arc<MongoPool>,
}

pub async fn ensure_indexes(pool: Arc<MongoPool>) -> Result<(), AppError> {
    let mangas = pool.database().collection::<MangaModel>("mangas");
    let channels = pool.database().collection::<ChannelModel>("channels");

    mangas
        .create_index(
            IndexModel::builder()
                .keys(doc! { "url": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await
        .map_err(|e| AppError::Infrastructure(format!("create manga index failed: {e}")))?;

    channels
        .create_index(
            IndexModel::builder()
                .keys(doc! { "guild_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await
        .map_err(|e| AppError::Infrastructure(format!("create channel index failed: {e}")))?;

    Ok(())
}

impl MongoChannelRepository {
    pub fn new(pool: Arc<MongoPool>) -> Self {
        Self { pool }
    }

    fn collection(&self) -> Collection<ChannelModel> {
        self.pool.database().collection("channels")
    }
}

#[async_trait]
impl ChannelRepository for MongoChannelRepository {
    async fn upsert_for_guild(&self, channel: &ChannelSubscription) -> Result<(), AppError> {
        self.collection()
            .update_one(
                doc! { "guild_id": &channel.guild_id },
                doc! {
                    "$set": {
                        "channel_id": &channel.channel_id,
                        "guild_id": &channel.guild_id,
                        "guild_name": &channel.guild_name,
                        "channel_name": &channel.channel_name,
                        "updated_at": BsonDateTime::from_system_time(channel.updated_at.into()),
                    },
                    "$setOnInsert": {
                        "created_at": BsonDateTime::from_system_time(channel.created_at.into())
                    }
                },
            )
            .upsert(true)
            .await
            .map_err(|e| AppError::Infrastructure(format!("upsert channel failed: {e}")))?;
        Ok(())
    }

    async fn list_by_guild(&self, guild_id: &str) -> Result<Vec<ChannelSubscription>, AppError> {
        let mut cursor = self
            .collection()
            .find(doc! { "guild_id": guild_id })
            .await
            .map_err(|e| AppError::Infrastructure(format!("list channels by guild failed: {e}")))?;
        let mut result = Vec::new();
        while let Some(item) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Infrastructure(format!("iterate channels failed: {e}")))?
        {
            result.push(item.into_domain());
        }
        Ok(result)
    }

    async fn list_all(&self) -> Result<Vec<ChannelSubscription>, AppError> {
        let mut cursor = self
            .collection()
            .find(doc! {})
            .await
            .map_err(|e| AppError::Infrastructure(format!("list channels failed: {e}")))?;
        let mut result = Vec::new();
        while let Some(item) = cursor
            .try_next()
            .await
            .map_err(|e| AppError::Infrastructure(format!("iterate channels failed: {e}")))?
        {
            result.push(item.into_domain());
        }
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MangaModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    url: String,
    latest_chapter: i32,
    latest_chapter_url: String,
    image_url: Option<String>,
    created_at: BsonDateTime,
    updated_at: BsonDateTime,
}

impl MangaModel {
    fn from_domain(value: &MangaTracking) -> Self {
        Self {
            id: None,
            title: value.title.clone(),
            url: value.url.clone(),
            latest_chapter: value.latest_chapter,
            latest_chapter_url: value.latest_chapter_url.clone(),
            image_url: value.image_url.clone(),
            created_at: BsonDateTime::from_system_time(value.created_at.into()),
            updated_at: BsonDateTime::from_system_time(value.updated_at.into()),
        }
    }

    fn into_domain(self) -> MangaTracking {
        MangaTracking {
            title: self.title,
            url: self.url,
            latest_chapter: self.latest_chapter,
            latest_chapter_url: self.latest_chapter_url,
            image_url: self.image_url,
            created_at: DateTime::<Utc>::from(self.created_at.to_system_time()),
            updated_at: DateTime::<Utc>::from(self.updated_at.to_system_time()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    channel_id: String,
    guild_id: String,
    guild_name: String,
    channel_name: String,
    created_at: BsonDateTime,
    updated_at: BsonDateTime,
}

impl ChannelModel {
    fn into_domain(self) -> ChannelSubscription {
        ChannelSubscription {
            channel_id: self.channel_id,
            guild_id: self.guild_id,
            guild_name: self.guild_name,
            channel_name: self.channel_name,
            created_at: DateTime::<Utc>::from(self.created_at.to_system_time()),
            updated_at: DateTime::<Utc>::from(self.updated_at.to_system_time()),
        }
    }
}
