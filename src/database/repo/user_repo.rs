use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set, sea_query::OnConflict,
};

use crate::{bot::enums::user_type::UserIdentity, database::models::user};

pub struct UserRepo {
    pub db: DatabaseConnection,
}

impl UserRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        UserRepo {
            db,
        }
    }

    #[inline]
    pub async fn get(&self, user: UserIdentity) -> Result<Option<user::Model>, DbErr> {
        match user {
            UserIdentity::Id(user_id) => {
                user::Entity::find_by_id(user_id)
                    .one(&self.db)
                    .await
            }
            UserIdentity::Username(username) => {
                user::Entity::find()
                    .filter(user::Column::Username.eq(username))
                    .one(&self.db)
                    .await
            }
        }
    }

    #[inline]
    pub async fn insert(
        &self,
        user_id: i64,
        username: Option<String>,
        full_name: String,
    ) -> Result<user::Model, DbErr> {
        let active_model = user::ActiveModel {
            id: Set(user_id),
            username: Set(username),
            full_name: Set(full_name),
        };

        user::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(user::Column::Id)
                    .update_columns([user::Column::Username, user::Column::FullName])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
    }
}
