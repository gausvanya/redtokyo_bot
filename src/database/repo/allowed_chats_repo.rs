use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult,
    IntoActiveModel, Set,
};

use crate::database::models::{allowed_chats, allowed_chats::ActiveModel};

pub struct AllowedChatsRepo {
    pub db: DatabaseConnection,
}

impl AllowedChatsRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
        }
    }

    #[inline]
    pub async fn get(&self, chat_id: i64) -> Result<Option<allowed_chats::Model>, DbErr> {
        allowed_chats::Entity::find_by_id(chat_id)
            .one(&self.db)
            .await
    }

    #[inline]
    pub async fn delete(&self, allowed_chat: allowed_chats::Model) -> Result<DeleteResult, DbErr> {
        allowed_chat
            .into_active_model()
            .delete(&self.db)
            .await
    }

    #[inline]
    pub async fn insert(&self, chat_id: i64) -> Result<InsertResult<ActiveModel>, DbErr> {
        let active_model = ActiveModel {
            chat_id: Set(chat_id),
        };

        allowed_chats::Entity::insert(active_model)
            .exec(&self.db)
            .await
    }
}
