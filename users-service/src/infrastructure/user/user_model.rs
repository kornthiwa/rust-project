use serde::Serialize;
use toasty::Model;

#[derive(Clone, Debug, Serialize, Model)]
pub struct UserModel {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub public_id: String,
    #[unique]
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserModel> for crate::domain::user::entity::User {
    fn from(model: UserModel) -> Self {
        Self {
            id: model.id,
            public_id: model.public_id,
            email: model.email,
            display_name: model.display_name,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
