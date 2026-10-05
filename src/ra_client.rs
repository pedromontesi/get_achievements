use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{config::Config, error::AppError};

pub struct RaClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    pub username: String,
    ttl: Duration,
    cache: Mutex<HashMap<String, (Instant, Value)>>,
}

impl RaClient {
    pub fn new(config: &Config) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("get_achievements_api/0.1")
            .build()
            .expect("falha ao criar o cliente HTTP");

        Self {
            http,
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            username: config.username.clone(),
            ttl: Duration::from_secs(config.cache_seconds),
            cache: Mutex::new(HashMap::new()),
        }
    }
    
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        extra: &[(&str, String)],
    ) -> Result<T, AppError> {
        let cache_key = format!("{endpoint}?{extra:?}");

        if let Some(value) = self.cached(&cache_key) {
            return serde_json::from_value(value).map_err(|_| AppError::BadData);
        }

        let mut query: Vec<(&str, String)> =
            vec![("y", self.api_key.clone()), ("u", self.username.clone())];
        query.extend(extra.iter().cloned());

        let response = self
            .http
            .get(format!("{}/API/{}.php", self.base_url, endpoint))
            .query(&query)
            .send()
            .await
            // `without_url` é essencial: a URL do erro contém a API_KEY.
            .map_err(|e| {
                eprintln!("falha de rede em {endpoint}: {}", e.without_url());
                AppError::Network
            })?;

        let status = response.status();
        if !status.is_success() {
            eprintln!("{endpoint} respondeu {status}");
            return Err(AppError::Upstream(status.as_u16()));
        }

        let value: Value = response.json().await.map_err(|e| {
            eprintln!("resposta de {endpoint} não é JSON: {}", e.without_url());
            AppError::BadData
        })?;

        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(cache_key, (Instant::now(), value.clone()));

        serde_json::from_value(value).map_err(|e| {
            eprintln!("formato inesperado em {endpoint}: {e}");
            AppError::BadData
        })
    }

    fn cached(&self, key: &str) -> Option<Value> {
        let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .get(key)
            .filter(|(saved_at, _)| saved_at.elapsed() < self.ttl)
            .map(|(_, value)| value.clone())
    }
}
