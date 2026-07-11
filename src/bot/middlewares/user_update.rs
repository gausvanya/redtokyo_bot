use crate::database::repo::user_repo::UserRepo;
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::LazyLock;
use std::time::Duration;
use telers::Request;
use telers::errors::EventErrorKind;
use telers::event::EventReturn;
use telers::middlewares::outer::{Middleware, MiddlewareResponse};

static USER_CACHE: LazyLock<Cache<i64, (Option<String>, String)>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(3600))
        .build()
});

#[derive(Clone)]
pub struct UpdateUserMiddleware;

impl<Client> Middleware<Client> for UpdateUserMiddleware
where
    Client: Send + Sync + 'static,
{
    async fn call(
        &mut self,
        request: Request<Client>,
    ) -> Result<MiddlewareResponse<Client>, EventErrorKind> {
        let (user_id, username, full_name) = if let Some(u) = request.update.from() {
            (
                u.id,
                u.username.as_ref().map(|s| s.to_string()),
                format!(
                    "{} {}",
                    u.first_name,
                    u.last_name.as_deref().unwrap_or_default()
                ),
            )
        } else if let Some(c) = request.update.sender_chat() {
            (
                c.id(),
                c.username().map(|s| s.to_string()),
                c.title().unwrap_or_default().to_string(),
            )
        } else {
            if let Some(c) = request.update.chat() {
                (
                    c.id(),
                    c.username().map(|s| s.to_string()),
                    format!(
                        "{} {}",
                        c.first_name().unwrap_or_default(),
                        c.last_name().unwrap_or_default()
                    ),
                )
            } else {
                (0, None, "Unknown".to_string())
            }
        };

        let is_actual = if let Some(cached_data) = USER_CACHE.get(&user_id).await {
            cached_data.0 == username && cached_data.1 == full_name
        } else {
            false
        };

        if !is_actual {
            let db = request.extensions.get::<DatabaseConnection>().unwrap();

            let user_repo = UserRepo::new(db.clone());

            let _ = user_repo.insert(user_id, username, full_name).await;
        }

        Ok((request, EventReturn::default()))
    }
}
