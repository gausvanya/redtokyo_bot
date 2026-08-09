use std::env;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Config {
    pub bot_token: Box<str>,
    pub database_url: Box<str>,
    pub iris_api_id: i64,
    pub iris_api_token: Box<str>,
}

impl Config {
    fn load() -> Self {
        dotenvy::dotenv().ok();

        Self {
            bot_token: env::var("BOT_TOKEN")
                .expect("bot token is not set.")
                .into_boxed_str(),
            database_url: env::var("DATABASE_URL")
                .expect("database url is not set.")
                .into_boxed_str(),
            iris_api_id: env::var("IRIS_API_ID")
                .expect("iris api id is not set.")
                .parse::<i64>()
                .expect("iris api id is not integer."),
            iris_api_token: env::var("IRIS_API_TOKEN")
                .expect("iris api token is not set.")
                .into_boxed_str(),
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}
