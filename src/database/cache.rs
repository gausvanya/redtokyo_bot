use crate::bot::middlewares::media_group::MediaItem;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub struct SummonPayload {
    pub creator_id: i64,
    pub msg_ids: Vec<i64>,
}

pub static SUMMON_CACHE: LazyLock<Cache<String, SummonPayload>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(86400))
        .build()
});

pub static WARN_CACHE: LazyLock<Cache<String, u32>> = LazyLock::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(1800))
        .build()
});

pub static MEDIA_GROUP_CACHE: LazyLock<Cache<String, Arc<Mutex<Vec<MediaItem>>>>> =
    LazyLock::new(|| {
        Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .build()
    });

pub static RAID_CACHE: LazyLock<Cache<String, Vec<i64>>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(15))
        .build()
});
