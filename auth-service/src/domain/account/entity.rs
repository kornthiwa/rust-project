use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: u64,
    pub public_id: String,
    pub username: String,
    pub password_hash: String,
    pub status: String,
    pub failed_login_attempts: i32,
    pub locked_until: Option<String>,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}