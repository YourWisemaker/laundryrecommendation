use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub openrouter_api_key: String,
    pub openrouter_base_url: String,
    pub or_model: String,
    pub openweather_api_key: String,
    pub openweather_base_url: String,
    pub openweather_onecall_path: String,
    pub openweather_forecast3h_path: String,
    pub openweather_geocode_direct_path: String,
    pub openweather_geocode_reverse_path: String,
    pub app_timezone: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY not set"))?,
            openrouter_base_url: env::var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string()),
            or_model: env::var("OR_MODEL")
                .unwrap_or_else(|_| "deepseek/deepseek-chat-v3-0324:free".to_string()),
            openweather_api_key: env::var("OPENWEATHER_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENWEATHER_API_KEY not set"))?,
            openweather_base_url: env::var("OPENWEATHER_BASE_URL")
                .unwrap_or_else(|_| "https://api.openweathermap.org".to_string()),
            openweather_onecall_path: env::var("OPENWEATHER_ONECALL_PATH")
                .unwrap_or_else(|_| "/data/3.0/onecall".to_string()),
            openweather_forecast3h_path: env::var("OPENWEATHER_FORECAST3H_PATH")
                .unwrap_or_else(|_| "/data/2.5/forecast".to_string()),
            openweather_geocode_direct_path: env::var("OPENWEATHER_GEOCODE_DIRECT_PATH")
                .unwrap_or_else(|_| "/geo/1.0/direct".to_string()),
            openweather_geocode_reverse_path: env::var("OPENWEATHER_GEOCODE_REVERSE_PATH")
                .unwrap_or_else(|_| "/geo/1.0/reverse".to_string()),
            app_timezone: env::var("APP_TIMEZONE")
                .unwrap_or_else(|_| "Asia/Jakarta".to_string()),
        })
    }
}