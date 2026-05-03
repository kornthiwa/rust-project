use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::application::error::AppError;
use crate::application::ports::{AuthEvent, AuthEventPublisher};
use crate::domain::account::entity::Account;
use crate::domain::account::repository::AccountRepository;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub access_token: String,
    pub token_type: String,
    pub expires_in_seconds: i64,
}

pub struct AuthService {
    repository: Arc<dyn AccountRepository>,
    jwt_secret: String,
    jwt_expiration_seconds: i64,
    auth_event_publisher: Arc<dyn AuthEventPublisher>,
}

impl AuthService {
    pub fn new(
        repository: Arc<dyn AccountRepository>,
        jwt_secret: String,
        jwt_expiration_seconds: i64,
        auth_event_publisher: Arc<dyn AuthEventPublisher>,
    ) -> Self {
        Self {
            repository,
            jwt_secret,
            jwt_expiration_seconds,
            auth_event_publisher,
        }
    }

    pub async fn register(&self, username: String, password: String) -> Result<Account, AppError> {
        if username.trim().is_empty() {
            return Err(AppError::validation(
                "invalid_input",
                "username is required",
            ));
        }
        if password.trim().is_empty() {
            return Err(AppError::validation(
                "invalid_input",
                "password is required",
            ));
        }

        let existing = self
            .repository
            .find_by_username(&username)
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?;
        if existing.is_some() {
            return Err(AppError::conflict(
                "username_taken",
                "username already exists",
            ));
        }

        let password_hash = hash_password(&password)
            .map_err(|_| AppError::validation("invalid_input", "invalid password"))?;
        let account = self
            .repository
            .create(
                username,
                password_hash,
                String::from("active"),
                0,
                None,
                None,
            )
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?;


        let event = AuthEvent::user_registered(account.id, account.username.clone());
        if let Err(e) = self.auth_event_publisher.publish(event).await {
            tracing::warn!(error = %e, "failed to publish user_registered event");
        }

        Ok(account)
    }

    pub async fn login(&self, username: String, password: String) -> Result<LoginResult, AppError> {
        if username.trim().is_empty() || password.trim().is_empty() {
            return Err(AppError::validation(
                "invalid_input",
                "username and password are required",
            ));
        }

        let account = self
            .repository
            .find_by_username(&username)
            .await
            .map_err(|err| AppError::internal_with_source("repository_error", err.to_string()))?
            .ok_or_else(|| AppError::unauthorized("invalid_credentials", "invalid credentials"))?;

        let is_password_valid = verify_password(&password, &account.password_hash)
            .map_err(|_| AppError::unauthorized("invalid_credentials", "invalid credentials"))?;
        if !is_password_valid {
            return Err(AppError::unauthorized(
                "invalid_credentials",
                "invalid credentials",
            ));
        }

        let account_id = account.id;
        let username_for_event = account.username.clone();

        let claims = JwtClaims {
            sub: account.id.to_string(),
            username: account.username,
            exp: (Utc::now() + Duration::seconds(self.jwt_expiration_seconds)).timestamp() as usize,
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::internal("token_creation_failed"))?;

        let result = LoginResult {
            access_token,
            token_type: String::from("Bearer"),
            expires_in_seconds: self.jwt_expiration_seconds,
        };

        let event = AuthEvent::user_logged_in(
            account_id,
            username_for_event,
            self.jwt_expiration_seconds,
        );
        if let Err(e) = self.auth_event_publisher.publish(event).await {
            tracing::warn!(error = %e, "failed to publish user_logged_in event");
        }

        Ok(result)
    }

    pub fn verify_jwt(&self, token: &str) -> Result<JwtClaims, AppError> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::unauthorized("invalid_token", "invalid token"))
    }
}

fn hash_password(raw_password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(raw_password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(raw_password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(raw_password.as_bytes(), &parsed_hash)
        .is_ok())
}
