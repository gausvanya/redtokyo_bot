use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "allowed_chats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub chat_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::ChatId",
        to = "super::user::Column::Id"
    )]
    Chat,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Chat.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
