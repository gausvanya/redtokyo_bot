use crate::database::models::verbal_warns;
use chrono::NaiveDateTime;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult,
    IntoActiveModel, NotSet, QueryFilter, Set,
};
use sea_orm::{ColumnTrait, QueryOrder};

pub struct VerbalWarnsRepo {
    pub db: DatabaseConnection,
}

impl VerbalWarnsRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        VerbalWarnsRepo { db }
    }

    #[inline]
    pub async fn get_all_from_user(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Vec<verbal_warns::Model>, DbErr> {
        verbal_warns::Entity::find()
            .filter(verbal_warns::Column::ChatId.eq(chat_id))
            .filter(verbal_warns::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
    }

    #[inline]
    pub async fn delete(&self, warn: verbal_warns::Model) -> Result<DeleteResult, DbErr> {
        warn.into_active_model().delete(&self.db).await
    }

    #[inline]
    pub async fn insert(
        &self,
        chat_id: i64,
        user_id: i64,
        admin_id: i64,
        message_id: i64,
        reason: String,
        timestamp: NaiveDateTime,
    ) -> Result<InsertResult<verbal_warns::ActiveModel>, DbErr> {
        let active_model = verbal_warns::ActiveModel {
            id: NotSet,
            chat_id: Set(chat_id),
            user_id: Set(user_id),
            admin_id: Set(admin_id),
            message_id: Set(message_id),
            reason: Set(reason),
            timestamp: Set(timestamp),
        };

        verbal_warns::Entity::insert(active_model)
            .exec(&self.db)
            .await
    }

    #[inline]
    pub async fn get(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Option<verbal_warns::Model>, DbErr> {
        verbal_warns::Entity::find()
            .filter(verbal_warns::Column::ChatId.eq(chat_id))
            .filter(verbal_warns::Column::UserId.eq(user_id))
            .order_by_desc(verbal_warns::Column::Timestamp)
            .one(&self.db)
            .await
    }
}
