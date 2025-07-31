use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherFeatures {
    pub temp_c: f64,
    pub rh: f64,
    pub wind_ms: f64,
    pub cloud: f64,
    pub rain_p: f64,
    pub rain_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFeatures {
    pub f_temp: f64,
    pub f_hum: f64,
    pub f_wind: f64,
    pub f_cloud: f64,
    pub f_rain: f64,
    pub f_vpd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryingWeights {
    pub w0: f64,
    pub w1: f64,
    pub w2: f64,
    pub w3: f64,
    pub w4: f64,
    pub w5: f64,
    pub w6: f64,
}

impl Default for DryingWeights {
    fn default() -> Self {
        Self {
            w0: 0.0,
            w1: 0.25,
            w2: 0.25,
            w3: 0.20,
            w4: 0.10,
            w5: 0.15,
            w6: 0.25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryingScore {
    pub score: f64,
    pub unsafe_window: bool,
    pub features: NormalizedFeatures,
    pub raw: WeatherFeatures,
    pub vpd_kpa: f64,
}

pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

pub fn calculate_vpd_kpa(temp_c: f64, rh: f64) -> f64 {
    let es = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let e = es * (rh / 100.0);
    (es - e).max(0.0)
}

pub fn normalize_features(weather: &WeatherFeatures) -> (NormalizedFeatures, f64) {
    let vpd_kpa = calculate_vpd_kpa(weather.temp_c, weather.rh);
    
    let features = NormalizedFeatures {
        f_temp: clamp((weather.temp_c - 15.0) / 15.0, 0.0, 1.0),
        f_hum: 1.0 - (weather.rh / 100.0).powf(0.7),
        f_wind: clamp(weather.wind_ms / 6.0, 0.0, 1.0),
        f_cloud: 1.0 - clamp(weather.cloud, 0.0, 1.0),
        f_rain: 1.0 - clamp(weather.rain_p, 0.0, 1.0),
        f_vpd: clamp(vpd_kpa / 2.5, 0.0, 1.0),
    };
    
    (features, vpd_kpa)
}

pub fn calculate_drying_score(weather: &WeatherFeatures, weights: &DryingWeights) -> DryingScore {
    let (features, vpd_kpa) = normalize_features(weather);
    
    // Hard veto check
    let unsafe_window = weather.rain_p > 0.50 || weather.rain_mm > 0.2;
    
    if unsafe_window {
        return DryingScore {
            score: -1.0,
            unsafe_window: true,
            features,
            raw: weather.clone(),
            vpd_kpa,
        };
    }
    
    // Linear score calculation
    let mut score = weights.w0 
        + weights.w1 * features.f_temp
        + weights.w2 * features.f_hum
        + weights.w3 * features.f_wind
        + weights.w4 * features.f_cloud
        + weights.w5 * features.f_rain
        + weights.w6 * features.f_vpd;
    
    // Soft penalties
    if weather.temp_c < 18.0 {
        score -= 0.15;
    }
    if weather.wind_ms < 1.0 {
        score -= 0.10;
    }
    
    DryingScore {
        score,
        unsafe_window: false,
        features,
        raw: weather.clone(),
        vpd_kpa,
    }
}

// Online learning update using logistic regression with SGD
pub fn update_weights_sgd(
    weights: &mut DryingWeights,
    features: &NormalizedFeatures,
    feedback: f64, // 0.0 or 1.0
    learning_rate: f64, // η ≈ 0.05
    regularization: f64, // λ ≈ 1e-4
) {
    // Create feature vector x = [1, f_temp, f_hum, f_wind, f_cloud, f_rain, f_vpd]
    let x = vec![
        1.0,
        features.f_temp,
        features.f_hum,
        features.f_wind,
        features.f_cloud,
        features.f_rain,
        features.f_vpd,
    ];
    
    // Current weights vector
    let w = vec![
        weights.w0,
        weights.w1,
        weights.w2,
        weights.w3,
        weights.w4,
        weights.w5,
        weights.w6,
    ];
    
    // Calculate z = w·x
    let z: f64 = w.iter().zip(x.iter()).map(|(wi, xi)| wi * xi).sum();
    
    // Calculate sigmoid p = σ(z) = 1/(1+e^-z)
    let p = 1.0 / (1.0 + (-z).exp());
    
    // Update weights: w := w - η * (p - y) * x + 2λw
    let error = p - feedback;
    
    weights.w0 -= learning_rate * (error * x[0] + 2.0 * regularization * w[0]);
    weights.w1 -= learning_rate * (error * x[1] + 2.0 * regularization * w[1]);
    weights.w2 -= learning_rate * (error * x[2] + 2.0 * regularization * w[2]);
    weights.w3 -= learning_rate * (error * x[3] + 2.0 * regularization * w[3]);
    weights.w4 -= learning_rate * (error * x[4] + 2.0 * regularization * w[4]);
    weights.w5 -= learning_rate * (error * x[5] + 2.0 * regularization * w[5]);
    weights.w6 -= learning_rate * (error * x[6] + 2.0 * regularization * w[6]);
    
    // Bound weights to reasonable ranges
    weights.w0 = clamp(weights.w0, -0.5, 0.5);
    weights.w1 = clamp(weights.w1, 0.0, 0.5);
    weights.w2 = clamp(weights.w2, 0.0, 0.5);
    weights.w3 = clamp(weights.w3, 0.0, 0.5);
    weights.w4 = clamp(weights.w4, 0.0, 0.3);
    weights.w5 = clamp(weights.w5, 0.0, 0.3);
    weights.w6 = clamp(weights.w6, 0.0, 0.5);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vpd_calculation() {
        let vpd = calculate_vpd_kpa(25.0, 60.0);
        assert!(vpd > 0.0);
        assert!(vpd < 5.0);
    }
    
    #[test]
    fn test_feature_normalization() {
        let weather = WeatherFeatures {
            temp_c: 25.0,
            rh: 60.0,
            wind_ms: 3.0,
            cloud: 0.3,
            rain_p: 0.1,
            rain_mm: 0.0,
        };
        
        let (features, _) = normalize_features(&weather);
        
        assert!(features.f_temp >= 0.0 && features.f_temp <= 1.0);
        assert!(features.f_hum >= 0.0 && features.f_hum <= 1.0);
        assert!(features.f_wind >= 0.0 && features.f_wind <= 1.0);
        assert!(features.f_cloud >= 0.0 && features.f_cloud <= 1.0);
        assert!(features.f_rain >= 0.0 && features.f_rain <= 1.0);
        assert!(features.f_vpd >= 0.0 && features.f_vpd <= 1.0);
    }
    
    #[test]
    fn test_hard_veto() {
        let weather = WeatherFeatures {
            temp_c: 25.0,
            rh: 60.0,
            wind_ms: 3.0,
            cloud: 0.3,
            rain_p: 0.6, // > 0.50, should trigger veto
            rain_mm: 0.0,
        };
        
        let weights = DryingWeights::default();
        let score = calculate_drying_score(&weather, &weights);
        
        assert!(score.unsafe_window);
        assert_eq!(score.score, -1.0);
    }
    
    #[test]
    fn test_sgd_update() {
        let mut weights = DryingWeights::default();
        let original_w1 = weights.w1;
        
        let features = NormalizedFeatures {
            f_temp: 0.5,
            f_hum: 0.6,
            f_wind: 0.4,
            f_cloud: 0.7,
            f_rain: 0.9,
            f_vpd: 0.5,
        };
        
        update_weights_sgd(&mut weights, &features, 1.0, 0.05, 1e-4);
        
        // Weights should have changed
        assert_ne!(weights.w1, original_w1);
        
        // Weights should be within bounds
        assert!(weights.w1 >= 0.0 && weights.w1 <= 0.5);
    }
}