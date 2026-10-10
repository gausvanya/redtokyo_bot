use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    IntoActiveModel, QueryFilter, Set, sea_query::OnConflict,
};

use crate::database::models::bans;

pub struct BansRepo {
    pub db: DatabaseConnection,
}

impl BansRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
        }
    }

    #[inline]
    pub async fn get(&self, user_id: i64) -> Result<Option<bans::Model>, DbErr> {
        bans::Entity::find()
            .filter(bans::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    // #[inline]
    // pub async fn get_all(&self) -> Result<Vec<(bans::Model, Vec<user::Model>)>, DbErr> {
    //     bans::Entity::find()
    //         .find_with_related(user::Entity)
    //         .all(&self.db)
    //         .await
    // }

    #[inline]
    pub async fn delete(&self, bans: bans::Model) -> Result<DeleteResult, DbErr> {
        bans.into_active_model()
            .delete(&self.db)
            .await
    }

    #[inline]
    pub async fn insert(
        &self,
        user_id: i64,
        admin_id: i64,
        reason: Option<String>,
    ) -> Result<bans::Model, DbErr> {
        let active_model = bans::ActiveModel {
            id: Default::default(),
            user_id: Set(user_id),
            admin_id: Set(admin_id),
            reason: Set(reason),
        };

        bans::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(bans::Column::UserId)
                    .update_columns([bans::Column::AdminId, bans::Column::Reason])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
    }
}
