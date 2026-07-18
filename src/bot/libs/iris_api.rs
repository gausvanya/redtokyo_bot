use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};
use std::collections::HashMap;

pub struct IrisAPI {
    api_id: i64,
    api_token: Box<str>,
    client: Client,
    base_url: Box<str>,
    api_version: Box<str>,
}

impl IrisAPI {
    pub fn new(api_id: i64, api_token: Box<str>) -> Self {
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

    async fn send_request(
        &self,
        method: &str,
        params: HashMap<String, String>,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let url = format!(
            "{}/v{}/{}_{}/{}",
            self.base_url, self.api_version, self.api_id, self.api_token, method
        );

        self.client
            .get(url)
            .query(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await
    }

    pub async fn get_user_reg(&self, user_id: i64) -> Result<serde_json::Value, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), user_id.to_string());

        self.send_request("user_info/reg", params).await
    }
}
