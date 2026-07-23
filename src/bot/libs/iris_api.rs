use crate::config::get_config;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT},
    Client,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use crate::database::cache::ORDER_BOOK_CACHE;

pub struct IrisAPI {
    api_id: i64,
    api_token: Box<str>,
    client: Client,
    base_url: Box<str>,
    api_version: Box<str>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetails {
    code: i32,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetails,
}

#[derive(Debug, thiserror::Error)]
pub enum IrisApiError {
    #[error("сетевая ошибка: {0}")]
    Request(#[from] reqwest::Error),

    #[error("ошибка API (код {code}): {description}")]
    Api { code: i32, description: String },

    #[error("неожиданный HTTP статус {0}: {1}")]
    BadStatus(reqwest::StatusCode, String),
}

#[derive(Debug, Deserialize)]
struct OrderBookEntry {
    #[allow(dead_code)]
    volume: f64,
    price: f64,
}

#[derive(Debug, Deserialize)]
struct OrderBookResult {
    buy: Vec<OrderBookEntry>,
    sell: Vec<OrderBookEntry>,
}

#[derive(Debug, Deserialize)]
pub struct OrderBookResponse {
    result: OrderBookResult,
}

#[derive(Debug, thiserror::Error)]
pub enum DuelRateError {
    #[error("iris api error: {0}")]
    Api(String),
    #[error("empty order book")]
    EmptyOrderBook,
    #[error("invalid mid price: {0}")]
    InvalidPrice(f64),
}


impl OrderBookResponse {
    pub fn best_buy(&self) -> Option<f64> {
        self.result.buy.first().map(|e| e.price)
    }

    pub fn best_sell(&self) -> Option<f64> {
        self.result.sell.first().map(|e| e.price)
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_buy(), self.best_sell()) {
            (Some(buy), Some(sell)) => Some((buy + sell) / 2.0),
            _ => None,
        }
    }
}

impl IrisAPI {
    pub fn new() -> Self {
        let cfg = get_config();
        let api_id = cfg.iris_api_id;
        let api_token = cfg.iris_api_token.clone();

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("MyApplicationAPI"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create reqwest client");

        Self {
            api_id,
            api_token,
            client,
            base_url: "https://iris-tg.ru/api".into(),
            api_version: "0.5".into(),
        }
    }

    async fn send_request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: HashMap<String, String>,
    ) -> Result<T, IrisApiError> {
        let url = format!(
            "{}/{}_{}/v{}/{}",
            self.base_url, self.api_id, self.api_token, self.api_version, method
        );

        let response = self.client
            .get(url)
            .query(&params)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            response.json::<T>().await.map_err(IrisApiError::Request)
        } else {
            let body_text = response.text().await.unwrap_or_default();

            if let Ok(err_payload) = serde_json::from_str::<ApiErrorResponse>(&body_text) {
                Err(IrisApiError::Api {
                    code: err_payload.error.code,
                    description: err_payload.error.description,
                })
            } else {
                Err(IrisApiError::BadStatus(status, body_text))
            }
        }
    }

    pub async fn get_user_reg(&self, user_id: i64) -> Result<serde_json::Value, IrisApiError> {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), user_id.to_string());

        self.send_request("user_info/reg", params).await
    }

    pub async fn get_order_book(&self) -> Result<OrderBookResponse, IrisApiError> {
        let params = HashMap::new();
        self.send_request("trade/orderbook", params).await
    }
}
