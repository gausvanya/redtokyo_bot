use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbConn, DbErr};

pub mod cache;
pub mod models;
pub mod repo;

#[inline]
async fn _migration(db: &DbConn) -> Result<(), DbErr> {
    db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS citext")
        .await?;

    db.get_schema_builder()
        .register(models::admins::Entity)
        .apply(db)
        .await?;

    Ok(())
}

#[inline]
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);

    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(false)
        .set_schema_search_path("public");

    let db = Database::connect(opt).await?;

    // _migration(&db).await?;
    Ok(db)
}
