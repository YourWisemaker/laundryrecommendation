use super::types::*;
use chrono::{DateTime, Duration, FixedOffset, Timelike};
use std::collections::HashMap;

pub fn merge_weather_data(
    onecall: Option<&OneCallResponse>,
    forecast3h: Option<&Forecast3hResponse>,
    timezone_offset: i32,
) -> Vec<HourlyData> {
    let mut merged_hours = Vec::new();
    let now = chrono::Utc::now();
    let target_offset = FixedOffset::east_opt(timezone_offset).unwrap_or(
        FixedOffset::east_opt(7 * 3600).unwrap() // Default to UTC+7
    );
    
    // Create a map for quick lookup of 3-hour forecast data
    let mut forecast3h_map: HashMap<i64, &Forecast3hItem> = HashMap::new();
    if let Some(forecast) = forecast3h {
        for item in &forecast.list {
            // Round to 3-hour boundaries
            let rounded_ts = (item.dt / (3 * 3600)) * (3 * 3600);
            forecast3h_map.insert(rounded_ts, item);
        }
    }
    
    // Generate hourly data for the next 7 days (168 hours)
    for hour_offset in 0..168 {
        let target_time = now + Duration::hours(hour_offset);
        let target_ts = target_time.timestamp();
        let target_3h_ts = (target_ts / (3 * 3600)) * (3 * 3600);
        
        let hourly_data = if hour_offset <= 120 && forecast3h_map.contains_key(&target_3h_ts) {
            // Prefer 3-hour forecast for exact 3-hour steps up to 120h
            let forecast_item = forecast3h_map[&target_3h_ts];
            let hours_data = HourlyData::from_forecast3h_item(forecast_item, timezone_offset);
            let hour_index = ((target_ts - target_3h_ts) / 3600) as usize;
            hours_data.get(hour_index).cloned().unwrap_or_else(|| {
                create_default_hourly_data(target_time.with_timezone(&target_offset))
            })
        } else if hour_offset <= 48 {
            // Use OneCall hourly for 0-48h
            if let Some(onecall) = onecall {
                if let Some(hourly) = onecall.hourly.get(hour_offset as usize) {
                    HourlyData::from(hourly)
                } else {
                    create_default_hourly_data(target_time.with_timezone(&target_offset))
                }
            } else {
                create_default_hourly_data(target_time.with_timezone(&target_offset))
            }
        } else {
            // Synthesize from daily up to day 7
            if let Some(onecall) = onecall {
                let day_index = hour_offset / 24;
                if let Some(daily) = onecall.daily.get(day_index as usize) {
                    let hour_of_day = (target_time.hour() as i32) % 24;
                    HourlyData::from_daily_synthesized(daily, timezone_offset, hour_of_day)
                } else {
                    create_default_hourly_data(target_time.with_timezone(&target_offset))
                }
            } else {
                create_default_hourly_data(target_time.with_timezone(&target_offset))
            }
        };
        
        merged_hours.push(hourly_data);
    }
    
    merged_hours
}

fn create_default_hourly_data(dt: DateTime<FixedOffset>) -> HourlyData {
    HourlyData {
        ts: dt,
        temp_c: 25.0,  // Default temperature
        rh: 60.0,      // Default humidity
        wind_ms: 2.0,  // Default wind speed
        cloud: 0.5,    // Default cloud cover
        rain_p: 0.0,   // No rain probability
        rain_mm: 0.0,  // No rain
    }
}

pub fn group_into_windows(
    hourly_data: &[HourlyData],
    step_hours: i32,
) -> Vec<WindowData> {
    let mut windows = Vec::new();
    
    for i in (0..hourly_data.len()).step_by(step_hours as usize) {
        let end_index = (i + step_hours as usize).min(hourly_data.len());
        let window_hours = &hourly_data[i..end_index];
        
        if window_hours.is_empty() {
            continue;
        }
        
        let start_time = window_hours[0].ts;
        let end_time = window_hours.last().unwrap().ts + Duration::hours(1);
        
        // Average the weather conditions over the window
        let avg_weather = average_weather_conditions(window_hours);
        
        windows.push(WindowData {
            id: format!("window_{}_{}", start_time.timestamp(), step_hours),
            start_time,
            end_time,
            weather: avg_weather,
            step_hours,
        });
    }
    
    windows
}

fn average_weather_conditions(hours: &[HourlyData]) -> crate::scoring::WeatherFeatures {
    let count = hours.len() as f64;
    
    let temp_sum: f64 = hours.iter().map(|h| h.temp_c).sum();
    let rh_sum: f64 = hours.iter().map(|h| h.rh).sum();
    let wind_sum: f64 = hours.iter().map(|h| h.wind_ms).sum();
    let cloud_sum: f64 = hours.iter().map(|h| h.cloud).sum();
    let rain_p_max: f64 = hours.iter().map(|h| h.rain_p).fold(0.0, f64::max);
    let rain_mm_sum: f64 = hours.iter().map(|h| h.rain_mm).sum();
    
    crate::scoring::WeatherFeatures {
        temp_c: temp_sum / count,
        rh: rh_sum / count,
        wind_ms: wind_sum / count,
        cloud: cloud_sum / count,
        rain_p: rain_p_max, // Use max rain probability in window
        rain_mm: rain_mm_sum, // Sum total expected rain
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowData {
    pub id: String,
    pub start_time: DateTime<FixedOffset>,
    pub end_time: DateTime<FixedOffset>,
    pub weather: crate::scoring::WeatherFeatures,
    pub step_hours: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_group_into_windows() {
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap());
        let hourly_data = vec![
            HourlyData {
                ts: now,
                temp_c: 25.0,
                rh: 60.0,
                wind_ms: 2.0,
                cloud: 0.3,
                rain_p: 0.1,
                rain_mm: 0.0,
            },
            HourlyData {
                ts: now + Duration::hours(1),
                temp_c: 26.0,
                rh: 58.0,
                wind_ms: 2.5,
                cloud: 0.2,
                rain_p: 0.0,
                rain_mm: 0.0,
            },
            HourlyData {
                ts: now + Duration::hours(2),
                temp_c: 27.0,
                rh: 55.0,
                wind_ms: 3.0,
                cloud: 0.1,
                rain_p: 0.0,
                rain_mm: 0.0,
            },
        ];
        
        let windows = group_into_windows(&hourly_data, 3);
        
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].step_hours, 3);
        assert!((windows[0].weather.temp_c - 26.0).abs() < 0.1); // Average of 25, 26, 27
        assert_eq!(windows[0].weather.rain_p, 0.1); // Max rain probability
    }
    
    #[test]
    fn test_average_weather_conditions() {
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap());
        let hours = vec![
            HourlyData {
                ts: now,
                temp_c: 20.0,
                rh: 60.0,
                wind_ms: 2.0,
                cloud: 0.3,
                rain_p: 0.1,
                rain_mm: 0.5,
            },
            HourlyData {
                ts: now + Duration::hours(1),
                temp_c: 30.0,
                rh: 40.0,
                wind_ms: 4.0,
                cloud: 0.1,
                rain_p: 0.3,
                rain_mm: 1.0,
            },
        ];
        
        let avg = average_weather_conditions(&hours);
        
        assert_eq!(avg.temp_c, 25.0); // (20 + 30) / 2
        assert_eq!(avg.rh, 50.0); // (60 + 40) / 2
        assert_eq!(avg.wind_ms, 3.0); // (2 + 4) / 2
        assert_eq!(avg.cloud, 0.2); // (0.3 + 0.1) / 2
        assert_eq!(avg.rain_p, 0.3); // max(0.1, 0.3)
        assert_eq!(avg.rain_mm, 1.5); // 0.5 + 1.0
    }
}