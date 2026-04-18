use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct UserNameInput {
    pub active: Option<bool>,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub active: bool,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


