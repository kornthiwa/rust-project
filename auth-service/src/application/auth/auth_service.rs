use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::application::auth::error::AppError;
use crate::domain::auth::entity::{CreateAccountInput, CreateAccountResult, LoginInput, LoginResult, MeResult};
use crate::domain::auth::error::DomainError;
use crate::domain::auth::repository::AuthRepository;

type Result<T> = std::result::Result<T, AppError>;
type Repository = Arc<dyn AuthRepository + Send + Sync>;

pub struct AuthService {
    repository: Repository,
    jwt_secret: String,
    jwt_exp_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    sub: String,
    username: String,
    exp: usize,
}

impl AuthService {
    pub fn new(repository: Repository, jwt_secret: String, jwt_exp_minutes: i64) -> Self {
        Self {
            repository,
            jwt_secret,
            jwt_exp_minutes,
        }
    }

    pub async fn create_account(&self, input: CreateAccountInput) -> Result<CreateAccountResult> {
        let hashed_password = Self::hash_password(&input.password)?;
        let created = self
            .repository
            .create_account(CreateAccountInput {
                username: input.username,
                password: hashed_password,
            })
            .await
            .map_err(AppError::from)?;

        Ok(CreateAccountResult {
            account_id: created.id,
            username: created.username,
            active: created.active,
        })
    }

    pub async fn login(&self, input: LoginInput) -> Result<LoginResult> {
        let account = self
            .repository
            .get_account_by_username(&input.username)
            .await
            .map_err(AppError::from)?;

        if !account.active {
            return Err(AppError::from(DomainError::InactiveAccount));
        }

        if !Self::verify_password(&input.password, &account.password)? {
            return Err(AppError::from(DomainError::InvalidCredentials));
        }

        let expires_at = Utc::now() + Duration::minutes(self.jwt_exp_minutes);
        let claims = AccessTokenClaims {
            sub: account.id.to_string(),
            username: account.username.clone(),
            exp: expires_at.timestamp() as usize,
        };
        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::Internal)?;

        Ok(LoginResult {
            account_id: account.id,
            username: account.username,
            is_authenticated: true,
            access_token,
            token_type: "Bearer".to_string(),
            expires_at,
        })
    }

    pub fn me_from_bearer_token(&self, authorization_header: &str) -> Result<MeResult> {
        let token = authorization_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let token_data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        let claims = token_data.claims;
        let account_id = claims.sub.parse::<i32>().map_err(|_| AppError::Unauthorized)?;
        let expires_at = chrono::DateTime::<Utc>::from_timestamp(claims.exp as i64, 0)
            .ok_or(AppError::Unauthorized)?;

        Ok(MeResult {
            account_id,
            username: claims.username,
            expires_at,
        })
    }

    fn hash_password(plain_password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(plain_password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::Internal)
    }

    fn verify_password(plain_password: &str, hashed_password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hashed_password).map_err(|_| AppError::Internal)?;
        Ok(Argon2::default()
            .verify_password(plain_password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
