use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub public_id: String,
    pub conversation_id: String,
    pub author_subject: String,
    pub body: String,
    pub created_at: String,
}
