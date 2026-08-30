use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use telers::{
    Request,
    errors::EventErrorKind,
    event::EventReturn,
    middlewares::outer::{Middleware, MiddlewareResponse},
};
use tokio::{sync::Mutex, time::sleep};

use crate::database::cache::MEDIA_GROUP_CACHE;

const QUIET_PERIOD: Duration = Duration::from_millis(900);
const MAX_WAIT: Duration = Duration::from_secs(8);

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

pub struct MediaGroupState {
    pub items: Vec<MediaItem>,
    pub last_update: Instant,
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

            let mutex = if let Some(m) = MEDIA_GROUP_CACHE
                .get(&mg_id_str)
                .await
            {
                m
            } else {
                let new_mutex = Arc::new(Mutex::new(MediaGroupState {
                    items: Vec::new(),
                    last_update: Instant::now(),
                }));
                MEDIA_GROUP_CACHE
                    .insert(mg_id_str.clone(), Arc::clone(&new_mutex))
                    .await;
                new_mutex
            };

            let new_item = if let Some(photo) = message
                .photo()
                .and_then(|p| p.last())
            {
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
                message
                    .document()
                    .map(|document| MediaItem {
                        file_id: document
                            .file_id
                            .to_string(),
                        kind: MediaKind::Video,
                    })
            };

            {
                let mut guard = mutex.lock().await;
                if let Some(item) = new_item
                    && !guard
                        .items
                        .iter()
                        .any(|i| i.file_id == item.file_id)
                {
                    guard.items.push(item);
                }
                guard.last_update = Instant::now();
            }

            let wait_started = Instant::now();
            loop {
                sleep(Duration::from_millis(200)).await;

                let last_update = {
                    let guard = mutex.lock().await;
                    guard.last_update
                };

                if last_update.elapsed() >= QUIET_PERIOD {
                    break;
                }
                if wait_started.elapsed() >= MAX_WAIT {
                    break;
                }
            }
        }

        Ok((request, EventReturn::default()))
    }
}
