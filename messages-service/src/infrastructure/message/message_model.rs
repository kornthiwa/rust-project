use serde::Serialize;
use toasty::Model;

#[derive(Clone, Debug, Serialize, Model)]
pub struct MessageModel {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub public_id: String,
    #[index]
    pub conversation_id: String,
    pub author_subject: String,
    pub body: String,
    pub created_at: String,
}

impl From<MessageModel> for crate::domain::message::entity::Message {
    fn from(model: MessageModel) -> Self {
        Self {
            id: model.id,
            public_id: model.public_id,
            conversation_id: model.conversation_id,
            author_subject: model.author_subject,
            body: model.body,
            created_at: model.created_at,
        }
    }
}
