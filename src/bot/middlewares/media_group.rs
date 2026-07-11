use crate::database::cache::MEDIA_GROUP_CACHE;
use std::sync::Arc;
use std::time::Duration;
use telers::Request;
use telers::errors::EventErrorKind;
use telers::event::EventReturn;
use telers::middlewares::outer::{Middleware, MiddlewareResponse};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Clone, Debug)]
pub enum MediaKind {
    Photo,
    Video,
}

#[derive(Clone, Debug)]
pub struct MediaItem {
    pub file_id: String,
    pub kind: MediaKind,
}

#[derive(Clone)]
pub struct MediaGroupMiddleware;

impl<Client> Middleware<Client> for MediaGroupMiddleware
where
    Client: Send + Sync + 'static,
{
    async fn call(
        &mut self,
        request: Request<Client>,
    ) -> Result<MiddlewareResponse<Client>, EventErrorKind> {
        let message = match request.update.message() {
            Some(msg) => msg,
            None => return Ok((request, EventReturn::default())),
        };

        if let Some(mg_id) = message.media_group_id() {
            let mg_id_str = mg_id.to_string();

            let mutex = if let Some(m) = MEDIA_GROUP_CACHE.get(&mg_id_str).await {
                m
            } else {
                let new_mutex = Arc::new(Mutex::new(Vec::new()));
                MEDIA_GROUP_CACHE
                    .insert(mg_id_str.clone(), Arc::clone(&new_mutex))
                    .await;
                new_mutex
            };

            let mut guard = mutex.lock().await;

            let new_item = if let Some(photo) = message.photo().and_then(|p| p.last()) {
                Some(MediaItem {
                    file_id: photo.file_id.to_string(),
                    kind: MediaKind::Photo,
                })
            } else if let Some(video) = message.video() {
                Some(MediaItem {
                    file_id: video.file_id.to_string(),
                    kind: MediaKind::Video,
                })
            } else {
                message.document().map(|document| MediaItem {
                    file_id: document.file_id.to_string(),
                    kind: MediaKind::Video,
                })
            };

            if let Some(item) = new_item
                && !guard.iter().any(|i| i.file_id == item.file_id)
            {
                guard.push(item);
            }

            sleep(Duration::from_millis(3000)).await;
        }

        Ok((request, EventReturn::default()))
    }
}
