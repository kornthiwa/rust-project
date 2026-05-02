use serde::Serialize;
use toasty::Model;

#[derive(Clone, Debug, Serialize, Model)]
pub struct AccountModel {
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

impl From<AccountModel> for crate::domain::account::entity::Account {
    fn from(m: AccountModel) -> Self {
        Self {
            id: m.id,
            public_id: m.public_id,
            username: m.username,
            password_hash: m.password_hash,
            status: m.status,
            failed_login_attempts: m.failed_login_attempts,
            locked_until: m.locked_until,
            last_login_at: m.last_login_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            deleted_at: m.deleted_at,
        }
    }
}
