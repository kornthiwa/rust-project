use serde::Serialize;
use toasty::Model;

#[derive(Clone, Debug, Serialize, Model)]
pub struct Account {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub public_id: String,
    #[unique]
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