use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Generate a unique window ID based on location and time
pub fn generate_window_id(lat: f64, lon: f64, start_time: DateTime<Utc>, duration_hours: u32) -> String {
    format!(
        "window_{:.4}_{:.4}_{}_{}h",
        lat,
        lon,
        start_time.timestamp(),
        duration_hours
    )
}

/// Validate latitude and longitude coordinates
pub fn validate_coordinates(lat: f64, lon: f64) -> Result<(), String> {
    if lat < -90.0 || lat > 90.0 {
        return Err(format!("Invalid latitude: {}. Must be between -90 and 90", lat));
    }
    if lon < -180.0 || lon > 180.0 {
        return Err(format!("Invalid longitude: {}. Must be between -180 and 180", lon));
    }
    Ok(())
}

/// Calculate distance between two coordinates using Haversine formula
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS_KM * c
}

/// Format duration in a human-readable way
pub fn format_duration(hours: u32) -> String {
    if hours == 0 {
        "0 hours".to_string()
    } else if hours == 1 {
        "1 hour".to_string()
    } else if hours < 24 {
        format!("{} hours", hours)
    } else {
        let days = hours / 24;
        let remaining_hours = hours % 24;
        if remaining_hours == 0 {
            if days == 1 {
                "1 day".to_string()
            } else {
                format!("{} days", days)
            }
        } else {
            if days == 1 {
                format!("1 day {} hours", remaining_hours)
            } else {
                format!("{} days {} hours", days, remaining_hours)
            }
        }
    }
}

/// Convert temperature between Celsius and Fahrenheit
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Convert wind speed between m/s and other units
pub fn ms_to_kmh(ms: f64) -> f64 {
    ms * 3.6
}

pub fn ms_to_mph(ms: f64) -> f64 {
    ms * 2.237
}

pub fn kmh_to_ms(kmh: f64) -> f64 {
    kmh / 3.6
}

pub fn mph_to_ms(mph: f64) -> f64 {
    mph / 2.237
}

/// Convert pressure between hPa and other units
pub fn hpa_to_inhg(hpa: f64) -> f64 {
    hpa * 0.02953
}

pub fn inhg_to_hpa(inhg: f64) -> f64 {
    inhg / 0.02953
}

/// Clamp a value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Linear interpolation between two values
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * clamp(t, 0.0, 1.0)
}

/// Calculate moving average for a slice of values
pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    if values.is_empty() || window_size == 0 {
        return Vec::new();
    }
    
    let mut result = Vec::new();
    
    for i in 0..values.len() {
        let start = if i >= window_size - 1 { i - window_size + 1 } else { 0 };
        let end = i + 1;
        let window = &values[start..end];
        let avg = window.iter().sum::<f64>() / window.len() as f64;
        result.push(avg);
    }
    
    result
}

/// Calculate exponential moving average
pub fn exponential_moving_average(values: &[f64], alpha: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    
    let mut result = Vec::with_capacity(values.len());
    let mut ema = values[0];
    result.push(ema);
    
    for &value in &values[1..] {
        ema = alpha * value + (1.0 - alpha) * ema;
        result.push(ema);
    }
    
    result
}

/// Round to specified decimal places
pub fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// Generate a random jitter value for retry delays
pub fn jitter(base_ms: u64, max_jitter_ms: u64) -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let jitter = rng.gen_range(0..=max_jitter_ms);
    base_ms + jitter
}

/// Parse timezone string and validate
pub fn parse_timezone(tz_str: &str) -> Result<chrono_tz::Tz, String> {
    tz_str.parse::<chrono_tz::Tz>()
        .map_err(|_| format!("Invalid timezone: {}", tz_str))
}

/// Convert UTC time to local timezone
pub fn utc_to_local(utc_time: DateTime<Utc>, timezone: &str) -> Result<DateTime<chrono_tz::Tz>, String> {
    let tz = parse_timezone(timezone)?;
    Ok(utc_time.with_timezone(&tz))
}

/// Sanitize user input strings
pub fn sanitize_string(input: &str, max_length: usize) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?-_()[]{}:;'\"".contains(*c))
        .take(max_length)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate email format (basic validation)
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5 && email.len() < 255
}

/// Generate a cache key for weather data
pub fn weather_cache_key(lat: f64, lon: f64, endpoint: &str) -> String {
    format!("weather_{}_{:.4}_{:.4}", endpoint, lat, lon)
}

/// Calculate cache TTL based on data freshness requirements
pub fn calculate_cache_ttl(data_type: &str) -> std::time::Duration {
    match data_type {
        "current" => std::time::Duration::from_secs(10 * 60), // 10 minutes
        "hourly" => std::time::Duration::from_secs(30 * 60), // 30 minutes
        "daily" => std::time::Duration::from_secs(2 * 60 * 60), // 2 hours
        "geocode" => std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
        _ => std::time::Duration::from_secs(30 * 60), // Default 30 minutes
    }
}

/// Rate limiting helper
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: HashMap<String, Vec<DateTime<Utc>>>,
    max_requests: usize,
    window_duration: chrono::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: chrono::Duration) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_duration,
        }
    }
    
    pub fn check_rate_limit(&mut self, key: &str) -> bool {
        let now = Utc::now();
        let cutoff = now - self.window_duration;
        
        // Clean old requests
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        requests.retain(|&timestamp| timestamp > cutoff);
        
        // Check if under limit
        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
    
    pub fn cleanup_old_entries(&mut self) {
        let now = Utc::now();
        let cutoff = now - self.window_duration;
        
        self.requests.retain(|_, timestamps| {
            timestamps.retain(|&timestamp| timestamp > cutoff);
            !timestamps.is_empty()
        });
    }
}

/// Error response helper
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: &str, code: &str) -> Self {
        Self {
            error: error.to_string(),
            code: code.to_string(),
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Success response helper
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_coordinates() {
        assert!(validate_coordinates(0.0, 0.0).is_ok());
        assert!(validate_coordinates(90.0, 180.0).is_ok());
        assert!(validate_coordinates(-90.0, -180.0).is_ok());
        assert!(validate_coordinates(91.0, 0.0).is_err());
        assert!(validate_coordinates(0.0, 181.0).is_err());
    }
    
    #[test]
    fn test_haversine_distance() {
        // Distance between New York and Los Angeles (approximately 3944 km)
        let distance = haversine_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((distance - 3944.0).abs() < 100.0); // Within 100km tolerance
    }
    
    #[test]
    fn test_temperature_conversion() {
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 0.01);
        assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < 0.01);
        assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < 0.01);
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0 hours");
        assert_eq!(format_duration(1), "1 hour");
        assert_eq!(format_duration(2), "2 hours");
        assert_eq!(format_duration(24), "1 day");
        assert_eq!(format_duration(25), "1 day 1 hours");
        assert_eq!(format_duration(48), "2 days");
    }
    
    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }
    
    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = moving_average(&values, 3);
        assert_eq!(avg.len(), 5);
        assert_eq!(avg[0], 1.0); // [1]
        assert_eq!(avg[1], 1.5); // [1, 2]
        assert_eq!(avg[2], 2.0); // [1, 2, 3]
        assert_eq!(avg[3], 3.0); // [2, 3, 4]
        assert_eq!(avg[4], 4.0); // [3, 4, 5]
    }
    
    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("Hello, World!", 20), "Hello, World!");
        assert_eq!(sanitize_string("<script>alert('xss')</script>", 20), "scriptalert'xss'script");
        assert_eq!(sanitize_string("Very long string that exceeds limit", 10), "Very long ");
    }
    
    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, chrono::Duration::seconds(60));
        
        assert!(limiter.check_rate_limit("user1"));
        assert!(limiter.check_rate_limit("user1"));
        assert!(!limiter.check_rate_limit("user1")); // Should be rate limited
        
        assert!(limiter.check_rate_limit("user2")); // Different user, should work
    }
}