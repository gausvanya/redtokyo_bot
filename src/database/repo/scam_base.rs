use crate::database::models::{scam_base, user};
use sea_orm::QueryFilter;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    IntoActiveModel, NotSet, Set,
};

pub struct ScamBaseRepo {
    pub db: DatabaseConnection,
}

impl ScamBaseRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        ScamBaseRepo { db }
    }

    #[inline]
    pub async fn get(
        &self,
        user_id: i64,
    ) -> Result<Option<(scam_base::Model, Option<user::Model>)>, DbErr> {
        scam_base::Entity::find()
            .filter(scam_base::Column::UserId.eq(user_id))
            .find_also_related(user::Entity)
            .one(&self.db)
            .await
    }

    #[inline]
    pub async fn update(
        &self,
        scam_base_model: scam_base::ActiveModel,
    ) -> Result<scam_base::Model, DbErr> {
        scam_base_model.update(&self.db).await
    }

    #[inline]
    pub async fn delete(&self, scam_base: scam_base::Model) -> Result<DeleteResult, DbErr> {
        scam_base.into_active_model().delete(&self.db).await
    }

    #[inline]
    pub async fn insert(
        &self,
        chat_id: i64,
        user_id: i64,
        message_id: i64,
        admin_id: i64,
        channel_chat_id: i64,
        channel_message_id: i64,
        reason: String,
        status: bool,
    ) -> Result<scam_base::Model, DbErr> {
        let active_model = scam_base::ActiveModel {
            id: NotSet,
            chat_id: Set(chat_id),
            user_id: Set(user_id),
            message_id: Set(message_id),
            admin_id: Set(admin_id),
            channel_chat_id: Set(channel_chat_id),
            channel_message_id: Set(channel_message_id),
            reason: Set(reason),
            status: Set(status),
        };

        scam_base::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(scam_base::Column::UserId)
                    .update_columns([
                        scam_base::Column::ChatId,
                        scam_base::Column::MessageId,
                        scam_base::Column::AdminId,
                        scam_base::Column::ChannelChatId,
                        scam_base::Column::ChannelMessageId,
                        scam_base::Column::Reason,
                        scam_base::Column::Status,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
    }
}
