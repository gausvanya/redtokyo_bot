use std::env;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct Config {
    pub bot_token: String,
    pub server_host: String,
    pub server_port: u16,
    pub webhook_url: String,
    pub webhook_path: String,
    pub secret_token: String,
    pub database_url: String,
    pub iris_api_id: i64,
    pub iris_api_token: String,
}

impl Config {
    fn load() -> Self {
        dotenvy::dotenv().ok();

        Self {
            bot_token: env::var("BOT_TOKEN").expect("bot token is not set."),
            server_host: env::var("SERVER_HOST").expect("server host is not set."),
            server_port: env::var("SERVER_PORT")
                .expect("server port is not set.")
                .parse::<u16>()
                .expect("server port is not integer."),
            webhook_url: env::var("WEBHOOK_URL").expect("webhook url is not set."),
            webhook_path: env::var("WEBHOOK_PATH").expect("webhook path is not set."),
            secret_token: env::var("SECRET_TOKEN").expect("secret token is not set."),
            database_url: env::var("DATABASE_URL").expect("database url is not set."),
            iris_api_id: env::var("IRIS_API_ID")
                .expect("iris api id is not set.")
                .parse::<i64>()
                .expect("iris api id is not integer."),
            iris_api_token: env::var("IRIS_API_TOKEN").expect("iris api token is not set."),
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}
