use crate::config::get_config;
use std::fs;
use std::path::Path;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

mod bot;
mod config;
mod database;

pub fn init_logger() {
    let log_dir = "logs";
    if !Path::new(log_dir).exists() {
        fs::create_dir_all(log_dir).expect("Не удалось создать папку логов");
    }

    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/error.log", log_dir))
        .expect("Не удалось открыть файл логов");

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .with_file(true);

    let file_layer = fmt::layer()
        .with_writer(file)
        .with_target(true)
        .with_line_number(true)
        .with_file(true)
        .with_filter(EnvFilter::new("error"));

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(EnvFilter::from_default_env())
        .init();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cfg = get_config();

    init_logger();

    let db = database::connect(&cfg.database_url).await?;
    bot::start(cfg, db).await?;

    Ok(())
}
