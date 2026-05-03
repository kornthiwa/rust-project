use std::sync::Arc;

use serde::Deserialize;

use crate::application::error::AppError;
use crate::application::ports::{UserEvent, UserEventPublisher};
use crate::domain::user::entity::User;
use crate::domain::user::repository::UserRepository;

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub display_name: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub is_active: Option<bool>,
}

pub struct UserService {
    repository: Arc<dyn UserRepository>,
    user_event_publisher: Arc<dyn UserEventPublisher>,
}

impl UserService {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        user_event_publisher: Arc<dyn UserEventPublisher>,
    ) -> Self {
        Self {
            repository,
            user_event_publisher,
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, AppError> {
        self.repository
            .list()
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))
    }

    pub async fn get_user(&self, user_id: u64) -> Result<User, AppError> {
        let user =
            self.repository.find_by_id(user_id).await.map_err(|err| {
                AppError::internal_with_source("repository_error", err.to_string())
            })?;

        user.ok_or_else(|| AppError::not_found("user_not_found", "user not found"))
    }

    pub async fn create_user(&self, input: CreateUserInput) -> Result<User, AppError> {
        validate_email(&input.email)?;
        validate_display_name(&input.display_name)?;

        let user = self
            .repository
            .create(
                input.email.trim().to_string(),
                input.display_name.trim().to_string(),
                input.is_active.unwrap_or(true),
            )
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?;

        let event = UserEvent::user_created(
            user.id,
            user.public_id.clone(),
            user.email.clone(),
            user.display_name.clone(),
        );
        if let Err(e) = self.user_event_publisher.publish(event).await {
            tracing::warn!(error = %e, "failed to publish user_created event");
        }

        Ok(user)
    }

    pub async fn update_user(
        &self,
        user_id: u64,
        input: UpdateUserInput,
    ) -> Result<User, AppError> {
        if let Some(email) = input.email.as_ref() {
            validate_email(email)?;
        }
        if let Some(display_name) = input.display_name.as_ref() {
            validate_display_name(display_name)?;
        }

        let user = self
            .repository
            .update(
                user_id,
                input.email.map(|value| value.trim().to_string()),
                input.display_name.map(|value| value.trim().to_string()),
                input.is_active,
            )
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?;

        let user = user.ok_or_else(|| AppError::not_found("user_not_found", "user not found"))?;

        let event = UserEvent::user_updated(
            user.id,
            user.public_id.clone(),
            user.email.clone(),
            user.display_name.clone(),
        );
        if let Err(e) = self.user_event_publisher.publish(event).await {
            tracing::warn!(error = %e, "failed to publish user_updated event");
        }

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: u64) -> Result<(), AppError> {
        let is_deleted =
            self.repository.delete(user_id).await.map_err(|err| {
                AppError::internal_with_source("repository_error", err.to_string())
            })?;

        if !is_deleted {
            return Err(AppError::not_found("user_not_found", "user not found"));
        }

        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if !email.contains('@') || email.trim().is_empty() {
        return Err(AppError::validation("invalid_email", "email is invalid"));
    }
    Ok(())
}

fn validate_display_name(display_name: &str) -> Result<(), AppError> {
    if display_name.trim().is_empty() {
        return Err(AppError::validation(
            "invalid_display_name",
            "display name is required",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::{CreateUserInput, UserService};
    use crate::application::error::AppError;
    use crate::application::ports::{UserEvent, UserEventPublisher};
    use crate::domain::user::entity::User;
    use crate::domain::user::repository::UserRepository;

    struct MockUserEventPublisher;

    #[async_trait]
    impl UserEventPublisher for MockUserEventPublisher {
        async fn publish(&self, _event: UserEvent) -> Result<(), String> {
            Ok(())
        }
    }

    struct MockUserRepository;

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
            Ok(vec![sample_user()])
        }

        async fn find_by_id(&self, user_id: u64) -> Result<Option<User>, sqlx::Error> {
            Ok((user_id == 1).then(sample_user))
        }

        async fn create(
            &self,
            email: String,
            display_name: String,
            is_active: bool,
        ) -> Result<User, sqlx::Error> {
            Ok(User {
                id: 2,
                public_id: "user_2".to_string(),
                email,
                display_name,
                is_active,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            })
        }

        async fn update(
            &self,
            _user_id: u64,
            _email: Option<String>,
            _display_name: Option<String>,
            _is_active: Option<bool>,
        ) -> Result<Option<User>, sqlx::Error> {
            Ok(None)
        }

        async fn delete(&self, _user_id: u64) -> Result<bool, sqlx::Error> {
            Ok(true)
        }
    }

    fn sample_user() -> User {
        User {
            id: 1,
            public_id: "user_1".to_string(),
            email: "demo@example.com".to_string(),
            display_name: "Demo".to_string(),
            is_active: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn noop_publisher() -> Arc<dyn UserEventPublisher> {
        Arc::new(MockUserEventPublisher)
    }

    #[tokio::test]
    async fn create_user_success_returns_created_user() {
        let service = UserService::new(Arc::new(MockUserRepository), noop_publisher());
        let input = CreateUserInput {
            email: "new@example.com".to_string(),
            display_name: "New User".to_string(),
            is_active: Some(true),
        };

        let user = service.create_user(input).await.expect("should succeed");
        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.display_name, "New User");
    }

    #[tokio::test]
    async fn create_user_failure_invalid_email_returns_validation_error() {
        let service = UserService::new(Arc::new(MockUserRepository), noop_publisher());
        let input = CreateUserInput {
            email: "invalid".to_string(),
            display_name: "New User".to_string(),
            is_active: Some(true),
        };

        let error = service.create_user(input).await.expect_err("should fail");
        match error {
            AppError::Validation { code, .. } => assert_eq!(code, "invalid_email"),
            _ => panic!("unexpected error type"),
        }
    }
}
