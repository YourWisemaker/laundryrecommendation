pub mod openweather;
pub mod mock;
pub mod types;
pub mod merge;

use moka::future::Cache;
use std::time::Duration;
use types::*;

pub type ForecastCache = Cache<String, CachedForecastData>;

pub fn init_cache() -> ForecastCache {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(30 * 60)) // 30 minutes
        .build()
}

#[derive(Clone, Debug)]
pub struct CachedForecastData {
    pub onecall: Option<OneCallResponse>,
    pub forecast3h: Option<Forecast3hResponse>,
    pub merged_hours: Vec<HourlyData>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}