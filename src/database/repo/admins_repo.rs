use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    InsertResult, IntoActiveModel, QueryFilter, Set,
};

use crate::database::models::{admins, user};
pub struct AdminRepo {
    pub db: DatabaseConnection,
}

impl AdminRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
        }
    }

    #[inline]
    pub async fn get(&self, chat_id: i64, user_id: i64) -> Result<Option<admins::Model>, DbErr> {
        admins::Entity::find()
            .filter(admins::Column::ChatId.eq(chat_id))
            .filter(admins::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    #[inline]
    pub async fn get_all(
        &self,
        chat_id: i64,
    ) -> Result<Vec<(admins::Model, Vec<user::Model>)>, DbErr> {
        admins::Entity::find()
            .filter(admins::Column::ChatId.eq(chat_id))
            .find_with_related(user::Entity)
            .all(&self.db)
            .await
    }

    #[inline]
    pub async fn delete(&self, admins: admins::Model) -> Result<DeleteResult, DbErr> {
        admins
            .into_active_model()
            .delete(&self.db)
            .await
    }

    #[inline]
    pub async fn insert(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<InsertResult<admins::ActiveModel>, DbErr> {
        let active_model = admins::ActiveModel {
            id: Default::default(),
            chat_id: Set(chat_id),
            user_id: Set(user_id),
        };

        admins::Entity::insert(active_model)
            .exec(&self.db)
            .await
    }
}
