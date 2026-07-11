use crate::bot::utils::datetime::get_current_datetime;
use crate::database::models::verbal_warns;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use std::time::Duration;

pub async fn verbal_warns_clear_task(db: DatabaseConnection) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let now = get_current_datetime();

        let result = verbal_warns::Entity::delete_many()
            .filter(verbal_warns::Column::Timestamp.lte(now))
            .exec(&db)
            .await;

        if let Err(e) = result {
            tracing::error!("Ошибка при очистке устных предупреждений: {:?}", e);
        }
    }
}
