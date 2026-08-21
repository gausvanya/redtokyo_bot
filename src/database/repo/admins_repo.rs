use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult,
    IntoActiveModel, Set,
};

use crate::database::models::{admins, admins::ActiveModel, user};
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
    pub async fn get(&self, user_id: i64) -> Result<Option<admins::Model>, DbErr> {
        admins::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
    }

    #[inline]
    pub async fn get_all(&self) -> Result<Vec<(admins::Model, Vec<user::Model>)>, DbErr> {
        admins::Entity::find()
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
    pub async fn insert(&self, chat_id: i64) -> Result<InsertResult<ActiveModel>, DbErr> {
        let active_model = ActiveModel {
            user_id: Set(chat_id),
        };

        admins::Entity::insert(active_model)
            .exec(&self.db)
            .await
    }
}
