use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyData {
    pub ts: chrono::DateTime<chrono::FixedOffset>,
    pub temp_c: f64,
    pub rh: f64,
    pub wind_ms: f64,
    pub cloud: f64,
    pub rain_p: f64,
    pub rain_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneCallResponse {
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub timezone_offset: i32,
    pub hourly: Vec<OneCallHourly>,
    pub daily: Vec<OneCallDaily>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneCallHourly {
    pub dt: i64,
    pub temp: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub clouds: f64,
    pub pop: f64,
    pub rain: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneCallDaily {
    pub dt: i64,
    pub temp: OneCallDailyTemp,
    pub humidity: f64,
    pub wind_speed: f64,
    pub clouds: f64,
    pub pop: f64,
    pub rain: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneCallDailyTemp {
    pub day: f64,
    pub min: f64,
    pub max: f64,
    pub night: f64,
    pub eve: f64,
    pub morn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hResponse {
    pub cod: String,
    pub message: f64,
    pub cnt: i32,
    pub list: Vec<Forecast3hItem>,
    pub city: Forecast3hCity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hItem {
    pub dt: i64,
    pub main: Forecast3hMain,
    pub weather: Vec<Forecast3hWeather>,
    pub clouds: Forecast3hClouds,
    pub wind: Forecast3hWind,
    pub visibility: Option<i32>,
    pub pop: f64,
    pub rain: Option<HashMap<String, f64>>,
    pub snow: Option<HashMap<String, f64>>,
    pub sys: Forecast3hSys,
    pub dt_txt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hMain {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: f64,
    pub sea_level: Option<f64>,
    pub grnd_level: Option<f64>,
    pub humidity: f64,
    pub temp_kf: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hWeather {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hClouds {
    pub all: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hWind {
    pub speed: f64,
    pub deg: f64,
    pub gust: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hSys {
    pub pod: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hCity {
    pub id: i32,
    pub name: String,
    pub coord: Forecast3hCoord,
    pub country: String,
    pub population: Option<i32>,
    pub timezone: i32,
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast3hCoord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeocodeResponse {
    pub name: String,
    pub local_names: Option<HashMap<String, String>>,
    pub lat: f64,
    pub lon: f64,
    pub country: String,
    pub state: Option<String>,
}