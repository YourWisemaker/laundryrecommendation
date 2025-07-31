use super::types::*;
use crate::config::Config;
use chrono::Timelike;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum OpenWeatherError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("Rate limited, retry after: {0}s")]
    RateLimited(u64),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid coordinates")]
    InvalidCoordinates,
}

pub struct OpenWeatherClient {
    client: Client,
    config: Config,
}

impl OpenWeatherClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .user_agent("LaundryDayOptimizer/1.0")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn get_onecall(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<OneCallResponse, OpenWeatherError> {
        if !self.is_valid_coordinates(lat, lon) {
            return Err(OpenWeatherError::InvalidCoordinates);
        }

        let url = format!(
            "{}{}",
            self.config.openweather_base_url, self.config.openweather_onecall_path
        );

        let response = self
            .make_request_with_retry(&url, &[
                ("lat", &lat.to_string()),
                ("lon", &lon.to_string()),
                ("units", "metric"),
                ("exclude", "minutely,alerts"),
                ("appid", &self.config.openweather_api_key),
            ])
            .await?;

        let onecall: OneCallResponse = serde_json::from_value(response)?;
        Ok(onecall)
    }

    pub async fn get_forecast3h(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<Forecast3hResponse, OpenWeatherError> {
        if !self.is_valid_coordinates(lat, lon) {
            return Err(OpenWeatherError::InvalidCoordinates);
        }

        let url = format!(
            "{}{}",
            self.config.openweather_base_url, self.config.openweather_forecast3h_path
        );

        let response = self
            .make_request_with_retry(&url, &[
                ("lat", &lat.to_string()),
                ("lon", &lon.to_string()),
                ("units", "metric"),
                ("appid", &self.config.openweather_api_key),
            ])
            .await?;

        let forecast: Forecast3hResponse = serde_json::from_value(response)?;
        Ok(forecast)
    }

    pub async fn geocode_direct(&self, query: &str) -> Result<Vec<GeocodeResponse>, OpenWeatherError> {
        let url = format!(
            "{}{}",
            self.config.openweather_base_url, self.config.openweather_geocode_direct_path
        );

        let response = self
            .make_request_with_retry(&url, &[
                ("q", query),
                ("limit", "1"),
                ("appid", &self.config.openweather_api_key),
            ])
            .await?;

        let geocode: Vec<GeocodeResponse> = serde_json::from_value(response)?;
        Ok(geocode)
    }

    pub async fn geocode_reverse(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<Vec<GeocodeResponse>, OpenWeatherError> {
        if !self.is_valid_coordinates(lat, lon) {
            return Err(OpenWeatherError::InvalidCoordinates);
        }

        let url = format!(
            "{}{}",
            self.config.openweather_base_url, self.config.openweather_geocode_reverse_path
        );

        let response = self
            .make_request_with_retry(&url, &[
                ("lat", &lat.to_string()),
                ("lon", &lon.to_string()),
                ("limit", "1"),
                ("appid", &self.config.openweather_api_key),
            ])
            .await?;

        let geocode: Vec<GeocodeResponse> = serde_json::from_value(response)?;
        Ok(geocode)
    }

    async fn make_request_with_retry(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<Value, OpenWeatherError> {
        let mut retry_count = 0;
        let max_retries = 3;
        let mut delay = Duration::from_millis(1000);

        loop {
            let response = self.client.get(url).query(params).send().await?;

            match response.status() {
                reqwest::StatusCode::OK => {
                    let json: Value = response.json().await?;
                    return Ok(json);
                }
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    if retry_count >= max_retries {
                        return Err(OpenWeatherError::RateLimited(delay.as_secs()));
                    }

                    tracing::warn!(
                        "Rate limited by OpenWeather API, retrying in {}ms",
                        delay.as_millis()
                    );

                    sleep(delay).await;
                    delay = delay.mul_f32(2.0 + fastrand::f32() * 0.5); // Exponential backoff with jitter
                    retry_count += 1;
                }
                status => {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(OpenWeatherError::ApiError(format!(
                        "HTTP {}: {}",
                        status, error_text
                    )));
                }
            }
        }
    }

    fn is_valid_coordinates(&self, lat: f64, lon: f64) -> bool {
        lat >= -90.0 && lat <= 90.0 && lon >= -180.0 && lon <= 180.0
    }
}

// Convert OpenWeather data to internal format
impl From<&OneCallHourly> for HourlyData {
    fn from(hourly: &OneCallHourly) -> Self {
        let rain_mm = hourly
            .rain
            .as_ref()
            .and_then(|r| r.get("1h"))
            .copied()
            .unwrap_or(0.0);

        // Convert UTC timestamp to fixed offset
        let dt = chrono::DateTime::from_timestamp(hourly.dt, 0)
            .unwrap_or_default()
            .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).unwrap()); // Default to UTC+7

        Self {
            ts: dt,
            temp_c: hourly.temp,
            rh: hourly.humidity,
            wind_ms: hourly.wind_speed,
            cloud: hourly.clouds / 100.0, // Convert percentage to 0-1
            rain_p: hourly.pop,
            rain_mm,
        }
    }
}

impl HourlyData {
    pub fn from_forecast3h_item(item: &Forecast3hItem, timezone_offset: i32) -> Vec<Self> {
        let rain_mm = item
            .rain
            .as_ref()
            .and_then(|r| r.get("3h"))
            .copied()
            .unwrap_or(0.0);

        let base_dt = chrono::DateTime::from_timestamp(item.dt, 0)
            .unwrap_or_default()
            .with_timezone(&chrono::FixedOffset::east_opt(timezone_offset).unwrap_or(
                chrono::FixedOffset::east_opt(7 * 3600).unwrap()
            ));

        // Replicate 3-hour data to 3 individual hours
        (0..3)
            .map(|hour_offset| Self {
                ts: base_dt + chrono::Duration::hours(hour_offset),
                temp_c: item.main.temp,
                rh: item.main.humidity,
                wind_ms: item.wind.speed,
                cloud: item.clouds.all / 100.0,
                rain_p: item.pop,
                rain_mm: rain_mm / 3.0, // Distribute 3h rain over 3 hours
            })
            .collect()
    }

    pub fn from_daily_synthesized(
        daily: &OneCallDaily,
        timezone_offset: i32,
        hour_of_day: i32,
    ) -> Self {
        let base_dt = chrono::DateTime::from_timestamp(daily.dt, 0)
            .unwrap_or_default()
            .with_timezone(&chrono::FixedOffset::east_opt(timezone_offset).unwrap_or(
                chrono::FixedOffset::east_opt(7 * 3600).unwrap()
            ))
            .with_hour(hour_of_day as u32)
            .unwrap_or_default();

        // Apply diurnal adjustments
        let is_daylight = hour_of_day >= 6 && hour_of_day < 18;
        let temp_adjustment = if is_daylight { 1.0 } else { -1.0 };
        let rh_adjustment = if is_daylight { -5.0 } else { 5.0 };
        let cloud_adjustment = if is_daylight { -0.1 } else { 0.1 };

        Self {
            ts: base_dt,
            temp_c: daily.temp.day + temp_adjustment,
            rh: (daily.humidity + rh_adjustment).clamp(0.0, 100.0),
            wind_ms: daily.wind_speed,
            cloud: (daily.clouds / 100.0 + cloud_adjustment).clamp(0.0, 1.0),
            rain_p: daily.pop / 8.0, // Distribute daily pop over 8 bins
            rain_mm: daily.rain.unwrap_or(0.0) / 8.0, // Distribute daily rain over 8 bins
        }
    }
}