use crate::database::models::{garant, user};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, IntoActiveModel, Set,
};

pub struct GarantRepo {
    pub db: DatabaseConnection,
}

impl GarantRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        GarantRepo { db }
    }

    #[inline]
    pub async fn get(&self, user_id: i64) -> Result<Option<garant::Model>, DbErr> {
        garant::Entity::find_by_id(user_id).one(&self.db).await
    }
    #[inline]
    pub async fn get_all(&self) -> Result<Vec<(garant::Model, Vec<user::Model>)>, DbErr> {
        garant::Entity::find()
            .find_with_related(user::Entity)
            .all(&self.db)
            .await
    }

    #[inline]
    pub async fn delete(&self, garant: garant::Model) -> Result<DeleteResult, DbErr> {
        garant.into_active_model().delete(&self.db).await
    }

    #[inline]
    pub async fn insert(&self, user_id: i64, comment: String) -> Result<garant::Model, DbErr> {
        let active_model = garant::ActiveModel {
            user_id: Set(user_id),
            comment: Set(comment),
        };

        garant::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(garant::Column::UserId)
                    .update_columns([garant::Column::Comment])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
    }
}
