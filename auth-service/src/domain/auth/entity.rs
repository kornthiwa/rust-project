use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct CreateAccountInput {
    pub username: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AccountInfo {
    pub id: i32,
    pub active: bool,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginResult {
    pub account_id: i32,
    pub username: String,
    pub is_authenticated: bool,
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateAccountResult {
    pub account_id: i32,
    pub username: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeResult {
    pub account_id: i32,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}
