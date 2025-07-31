use super::types::*;
use chrono::{DateTime, Duration, FixedOffset, Timelike, Utc};
use std::collections::HashMap;

pub struct MockWeatherClient;

impl MockWeatherClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_onecall(&self, lat: f64, lon: f64) -> Result<OneCallResponse, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let timezone_offset = 7 * 3600; // UTC+7 for Thailand
        
        // Generate 48 hours of mock hourly data
        let hourly = (0..48)
            .map(|hour| {
                let base_temp = 25.0 + 5.0 * (hour as f64 * 0.26).sin(); // Diurnal temperature variation
                let base_humidity = 60.0 + 20.0 * (hour as f64 * 0.13).cos(); // Humidity variation
                let wind_speed = 2.0 + 3.0 * fastrand::f64(); // Random wind
                let clouds = 30.0 + 40.0 * fastrand::f64(); // Random clouds
                let pop = if hour % 8 == 0 { 0.3 } else { 0.1 }; // Occasional rain
                
                let mut rain = None;
                if pop > 0.2 {
                    let mut rain_map = HashMap::new();
                    rain_map.insert("1h".to_string(), 2.0 * fastrand::f64());
                    rain = Some(rain_map);
                }
                
                OneCallHourly {
                    dt: (now + Duration::hours(hour)).timestamp(),
                    temp: base_temp,
                    humidity: base_humidity.clamp(30.0, 90.0),
                    wind_speed,
                    clouds,
                    pop,
                    rain,
                }
            })
            .collect();
        
        // Generate 7 days of mock daily data
        let daily = (0..7)
            .map(|day| {
                let base_temp = 28.0 + 3.0 * (day as f64 * 0.5).sin();
                let humidity = 65.0 + 15.0 * fastrand::f64();
                let wind_speed = 2.5 + 2.0 * fastrand::f64();
                let clouds = 40.0 + 30.0 * fastrand::f64();
                let pop = if day % 3 == 0 { 0.4 } else { 0.2 };
                let rain = if pop > 0.3 { Some(5.0 * fastrand::f64()) } else { None };
                
                OneCallDaily {
                    dt: (now + Duration::days(day)).timestamp(),
                    temp: OneCallDailyTemp {
                        day: base_temp,
                        min: base_temp - 5.0,
                        max: base_temp + 5.0,
                        night: base_temp - 3.0,
                        eve: base_temp + 2.0,
                        morn: base_temp - 2.0,
                    },
                    humidity,
                    wind_speed,
                    clouds,
                    pop,
                    rain,
                }
            })
            .collect();
        
        Ok(OneCallResponse {
            lat,
            lon,
            timezone: "Asia/Bangkok".to_string(),
            timezone_offset,
            hourly,
            daily,
        })
    }

    pub async fn get_forecast3h(&self, lat: f64, lon: f64) -> Result<Forecast3hResponse, Box<dyn std::error::Error>> {
        let now = Utc::now();
        
        // Generate 5 days of 3-hour forecast data (40 items)
        let list: Vec<_> = (0..40)
            .map(|i| {
                let hours_ahead = i * 3;
                let target_time = now + Duration::hours(hours_ahead);
                
                let base_temp = 26.0 + 4.0 * (hours_ahead as f64 * 0.26).sin();
                let humidity = 65.0 + 20.0 * (hours_ahead as f64 * 0.13).cos();
                let wind_speed = 2.5 + 2.5 * fastrand::f64();
                let clouds = 35.0 + 35.0 * fastrand::f64();
                let pop = if hours_ahead % 24 == 0 { 0.35 } else { 0.15 };
                
                let mut rain = None;
                if pop > 0.25 {
                    let mut rain_map = HashMap::new();
                    rain_map.insert("3h".to_string(), 3.0 * fastrand::f64());
                    rain = Some(rain_map);
                }
                
                Forecast3hItem {
                    dt: target_time.timestamp(),
                    main: Forecast3hMain {
                        temp: base_temp,
                        feels_like: base_temp + 2.0,
                        temp_min: base_temp - 2.0,
                        temp_max: base_temp + 2.0,
                        pressure: 1013.0 + 10.0 * fastrand::f64(),
                        sea_level: Some(1013.0),
                        grnd_level: Some(1010.0),
                        humidity,
                        temp_kf: Some(0.0),
                    },
                    weather: vec![Forecast3hWeather {
                        id: if pop > 0.25 { 500 } else { 800 },
                        main: if pop > 0.25 { "Rain".to_string() } else { "Clear".to_string() },
                        description: if pop > 0.25 { "light rain".to_string() } else { "clear sky".to_string() },
                        icon: if pop > 0.25 { "10d".to_string() } else { "01d".to_string() },
                    }],
                    clouds: Forecast3hClouds { all: clouds },
                    wind: Forecast3hWind {
                        speed: wind_speed,
                        deg: 180.0 + 90.0 * fastrand::f64(),
                        gust: Some(wind_speed * 1.5),
                    },
                    visibility: Some(10000),
                    pop,
                    rain,
                    snow: None,
                    sys: Forecast3hSys {
                        pod: if target_time.hour() >= 6 && target_time.hour() < 18 { "d".to_string() } else { "n".to_string() },
                    },
                    dt_txt: target_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            })
            .collect();
        
        Ok(Forecast3hResponse {
            cod: "200".to_string(),
            message: 0.0,
            cnt: list.len() as i32,
            list,
            city: Forecast3hCity {
                id: 1609350,
                name: "Bangkok".to_string(),
                coord: Forecast3hCoord { lat, lon },
                country: "TH".to_string(),
                population: Some(8280925),
                timezone: 7 * 3600,
                sunrise: (now.with_hour(6).unwrap_or(now)).timestamp(),
                sunset: (now.with_hour(18).unwrap_or(now)).timestamp(),
            },
        })
    }

    pub async fn geocode_direct(&self, query: &str) -> Result<Vec<GeocodeResponse>, Box<dyn std::error::Error>> {
        // Mock geocoding responses for common cities
        let mock_locations = vec![
            ("Bangkok", 13.7563, 100.5018, "TH", Some("Bangkok".to_string())),
            ("Chiang Mai", 18.7883, 98.9853, "TH", Some("Chiang Mai".to_string())),
            ("Phuket", 7.8804, 98.3923, "TH", Some("Phuket".to_string())),
            ("Pattaya", 12.9236, 100.8825, "TH", Some("Chonburi".to_string())),
        ];
        
        let query_lower = query.to_lowercase();
        let result = mock_locations
            .into_iter()
            .find(|(name, _, _, _, _)| name.to_lowercase().contains(&query_lower))
            .map(|(name, lat, lon, country, state)| GeocodeResponse {
                name: name.to_string(),
                local_names: None,
                lat,
                lon,
                country: country.to_string(),
                state,
            })
            .unwrap_or_else(|| GeocodeResponse {
                name: "Unknown Location".to_string(),
                local_names: None,
                lat: 13.7563,
                lon: 100.5018,
                country: "TH".to_string(),
                state: Some("Bangkok".to_string()),
            });
        
        Ok(vec![result])
    }

    pub async fn geocode_reverse(&self, lat: f64, lon: f64) -> Result<Vec<GeocodeResponse>, Box<dyn std::error::Error>> {
        // Simple reverse geocoding mock
        let name = if (lat - 13.7563).abs() < 1.0 && (lon - 100.5018).abs() < 1.0 {
            "Bangkok"
        } else if (lat - 18.7883).abs() < 1.0 && (lon - 98.9853).abs() < 1.0 {
            "Chiang Mai"
        } else {
            "Unknown Location"
        };
        
        Ok(vec![GeocodeResponse {
            name: name.to_string(),
            local_names: None,
            lat,
            lon,
            country: "TH".to_string(),
            state: Some("Thailand".to_string()),
        }])
    }
}

// Generate realistic mock data for testing
pub fn generate_mock_hourly_data(hours: usize, start_time: DateTime<FixedOffset>) -> Vec<HourlyData> {
    (0..hours)
        .map(|hour| {
            let time = start_time + Duration::hours(hour as i64);
            let hour_of_day = time.hour() as f64;
            
            // Realistic diurnal patterns
            let temp_base = 25.0;
            let temp_amplitude = 8.0;
            let temp_phase = (hour_of_day - 6.0) * std::f64::consts::PI / 12.0;
            let temp_c = temp_base + temp_amplitude * temp_phase.sin();
            
            let rh_base = 70.0;
            let rh_amplitude = 25.0;
            let rh_phase = (hour_of_day - 12.0) * std::f64::consts::PI / 12.0;
            let rh = (rh_base - rh_amplitude * rh_phase.sin()).clamp(30.0_f64, 95.0_f64);
            
            let wind_ms = 1.5 + 2.0 * fastrand::f64();
            let cloud = 0.2 + 0.6 * fastrand::f64();
            
            // Occasional rain events
            let rain_p = if hour % 12 == 0 { 0.3 } else { 0.05 };
            let rain_mm = if rain_p > 0.2 { 1.0 + 3.0 * fastrand::f64() } else { 0.0 };
            
            HourlyData {
                ts: time,
                temp_c,
                rh,
                wind_ms,
                cloud,
                rain_p,
                rain_mm,
            }
        })
        .collect()
}