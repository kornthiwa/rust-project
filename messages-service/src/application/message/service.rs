use std::sync::Arc;

use serde::Deserialize;

use crate::application::error::AppError;
use crate::domain::message::entity::Message;
use crate::domain::message::repository::MessageRepository;

const MAX_BODY_LEN: usize = 16_384;

#[derive(Debug, Deserialize)]
pub struct CreateMessageInput {
    pub conversation_id: String,
    pub body: String,
}

pub struct MessageService {
    repository: Arc<dyn MessageRepository>,
}

impl MessageService {
    pub fn new(repository: Arc<dyn MessageRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_message(
        &self,
        author_subject: String,
        input: CreateMessageInput,
    ) -> Result<Message, AppError> {
        validate_conversation_id(&input.conversation_id)?;
        validate_body(&input.body)?;

        self.repository
            .create(
                input.conversation_id.trim().to_string(),
                author_subject,
                input.body.trim().to_string(),
            )
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))
    }

    pub async fn get_message(&self, message_id: u64) -> Result<Message, AppError> {
        let message = self
            .repository
            .find_by_id(message_id)
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?;

        message.ok_or_else(|| AppError::not_found("message_not_found", "message not found"))
    }

    pub async fn list_messages(&self, conversation_id: &str) -> Result<Vec<Message>, AppError> {
        validate_conversation_id(conversation_id)?;

        self.repository
            .list_by_conversation(conversation_id.trim())
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))
    }
}

fn validate_conversation_id(conversation_id: &str) -> Result<(), AppError> {
    if conversation_id.trim().is_empty() {
        return Err(AppError::validation(
            "invalid_conversation_id",
            "conversation id is required",
        ));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), AppError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "invalid_body",
            "message body is required",
        ));
    }
    if trimmed.len() > MAX_BODY_LEN {
        return Err(AppError::validation(
            "body_too_long",
            "message body exceeds maximum length",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{CreateMessageInput, MessageService};
    use crate::application::error::AppError;
    use crate::domain::message::entity::Message;
    use crate::domain::message::repository::MessageRepository;

    struct MockMessageRepository;

    #[async_trait]
    impl MessageRepository for MockMessageRepository {
        async fn create(
            &self,
            conversation_id: String,
            author_subject: String,
            body: String,
        ) -> toasty::Result<Message> {
            Ok(Message {
                id: 1,
                public_id: "msg_1".to_string(),
                conversation_id,
                author_subject,
                body,
                created_at: "2026-01-01T00:00:00Z".to_string(),
            })
        }

        async fn find_by_id(&self, _message_id: u64) -> toasty::Result<Option<Message>> {
            Ok(None)
        }

        async fn list_by_conversation(
            &self,
            _conversation_id: &str,
        ) -> toasty::Result<Vec<Message>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn create_message_success_returns_message() {
        let service = MessageService::new(Arc::new(MockMessageRepository));
        let input = CreateMessageInput {
            conversation_id: "conv-1".to_string(),
            body: "hello".to_string(),
        };

        let message = service
            .create_message("user-sub".to_string(), input)
            .await
            .expect("should succeed");
        assert_eq!(message.conversation_id, "conv-1");
        assert_eq!(message.author_subject, "user-sub");
        assert_eq!(message.body, "hello");
    }

    #[tokio::test]
    async fn create_message_failure_empty_body_returns_validation_error() {
        let service = MessageService::new(Arc::new(MockMessageRepository));
        let input = CreateMessageInput {
            conversation_id: "conv-1".to_string(),
            body: "   ".to_string(),
        };

        let error = service
            .create_message("user-sub".to_string(), input)
            .await
            .expect_err("should fail");
        match error {
            AppError::Validation { code, .. } => assert_eq!(code, "invalid_body"),
            _ => panic!("unexpected error type"),
        }
    }
}
