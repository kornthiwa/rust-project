use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error, FromRow, PgPool};
use uuid::Uuid;

use crate::domain::message::entity::Message;
use crate::domain::message::repository::MessageRepository;

#[derive(Debug, FromRow)]
struct MessageRow {
    id: i64,
    public_id: String,
    conversation_id: String,
    author_subject: String,
    body: String,
    created_at: String,
}

impl From<MessageRow> for Message {
    fn from(r: MessageRow) -> Self {
        Self {
            id: r.id as u64,
            public_id: r.public_id,
            conversation_id: r.conversation_id,
            author_subject: r.author_subject,
            body: r.body,
            created_at: r.created_at,
        }
    }
}

pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn create(
        &self,
        conversation_id: String,
        author_subject: String,
        body: String,
    ) -> Result<Message, Error> {
        let now = Self::now_timestamp();
        let public_id = Uuid::new_v4().to_string();
        let row = sqlx::query_as::<_, MessageRow>(
            r#"INSERT INTO message_models (public_id, conversation_id, author_subject, body, created_at)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, public_id, conversation_id, author_subject, body, created_at"#,
        )
        .bind(&public_id)
        .bind(&conversation_id)
        .bind(&author_subject)
        .bind(&body)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn find_by_id(&self, message_id: u64) -> Result<Option<Message>, Error> {
        let row = sqlx::query_as::<_, MessageRow>(
            r#"SELECT id, public_id, conversation_id, author_subject, body, created_at
               FROM message_models WHERE id = $1"#,
        )
        .bind(message_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list_by_conversation(&self, conversation_id: &str) -> Result<Vec<Message>, Error> {
        let rows = sqlx::query_as::<_, MessageRow>(
            r#"SELECT id, public_id, conversation_id, author_subject, body, created_at
               FROM message_models WHERE conversation_id = $1 ORDER BY created_at ASC"#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
