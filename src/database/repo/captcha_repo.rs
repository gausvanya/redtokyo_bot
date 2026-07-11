use crate::database::models::successful_captcha;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, NotSet, QueryFilter, Set,
};

pub struct CaptchaRepo {
    pub db: DatabaseConnection,
}

impl CaptchaRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        CaptchaRepo { db }
    }

    #[inline]
    pub async fn get(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Option<successful_captcha::Model>, DbErr> {
        successful_captcha::Entity::find()
            .filter(successful_captcha::Column::ChatId.eq(chat_id))
            .filter(successful_captcha::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    #[inline]
    pub async fn insert(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<InsertResult<successful_captcha::ActiveModel>, DbErr> {
        let active_model = successful_captcha::ActiveModel {
            id: NotSet,
            chat_id: Set(chat_id),
            user_id: Set(user_id),
        };

        successful_captcha::Entity::insert(active_model)
            .exec(&self.db)
            .await
    }
}
