use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    ai::{AiClient, FeedbackAnalysis},
    config::Config,
    database::{CreateFeedback, CreateUserPreferences, Database, FeedbackRecord, UserPreferences},
    forecast::{
        merge::{group_into_windows, merge_weather_data, WindowData},
        openweather::OpenWeatherClient,
        types::{GeocodeResponse, HourlyData},
    },
    scoring::{calculate_drying_score, DryingScore, WeatherFeatures},
};

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: Arc<Database>,
    pub weather_client: Arc<OpenWeatherClient>,
    pub ai_client: Arc<AiClient>,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct GeocodeQuery {
    pub q: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastQuery {
    pub lat: f64,
    pub lon: f64,
    pub hours: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DryingWindowsQuery {
    pub lat: f64,
    pub lon: f64,
    pub window_hours: Option<u32>,
    pub max_windows: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationQuery {
    pub lat: f64,
    pub lon: f64,
    pub user_id: Option<Uuid>,
    pub window_hours: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub user_id: Option<Uuid>,
    pub window_id: String,
    pub feedback_text: String,
    pub satisfaction_rating: Option<i32>,
    pub drying_result: Option<String>,
    pub weather_conditions: Option<WeatherConditions>,
    pub predicted_score: Option<f64>,
    pub actual_outcome: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WeatherConditions {
    pub temp_c: Option<f64>,
    pub humidity: Option<f64>,
    pub wind_ms: Option<f64>,
    pub rain_mm: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub window_data: WindowData,
    pub score: DryingScore,
    pub user_preferences: Option<UserPreferences>,
}

#[derive(Debug, Deserialize)]
pub struct AiRecommendationQuery {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ForecastResponse {
    pub location: LocationInfo,
    pub hourly_data: Vec<HourlyData>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct LocationInfo {
    pub lat: f64,
    pub lon: f64,
    pub name: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DryingWindowsResponse {
    pub location: LocationInfo,
    pub windows: Vec<DryingWindow>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct DryingWindow {
    pub id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_hours: u32,
    pub score: DryingScore,
    pub weather_summary: WeatherSummary,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct WeatherSummary {
    pub avg_temp_c: f64,
    pub avg_humidity: f64,
    pub avg_wind_ms: f64,
    pub total_rain_mm: f64,
    pub conditions: String,
}

#[derive(Debug, Serialize)]
pub struct AiRecommendationResponse {
    pub recommendation: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub location: LocationInfo,
    pub best_windows: Vec<DryingWindow>,
    pub ai_explanation: Option<String>,
    pub tips: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct FeedbackResponse {
    pub id: Uuid,
    pub analysis: Option<FeedbackAnalysis>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub factors: Vec<String>,
    pub tips: Vec<String>,
}

// Route handlers
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn geocode(
    State(state): State<AppState>,
    Query(params): Query<GeocodeQuery>,
) -> Result<Json<Vec<GeocodeResponse>>, StatusCode> {
    let _limit = params.limit.unwrap_or(5).min(10);
    
    // Check if this is reverse geocoding (lat/lon provided) or direct geocoding (q provided)
    if let (Some(lat), Some(lon)) = (params.lat, params.lon) {
        // Reverse geocoding
        match state.weather_client.geocode_reverse(lat, lon).await {
            Ok(results) => Ok(Json(results)),
            Err(e) => {
                tracing::error!("Reverse geocoding failed: {}", e);
                Err(StatusCode::BAD_REQUEST)
            }
        }
    } else if let Some(query) = params.q {
        // Direct geocoding
        match state.weather_client.geocode_direct(&query).await {
            Ok(results) => Ok(Json(results)),
            Err(e) => {
                tracing::error!("Direct geocoding failed: {}", e);
                Err(StatusCode::BAD_REQUEST)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn get_forecast(
    State(state): State<AppState>,
    Query(params): Query<ForecastQuery>,
) -> Result<Json<ForecastResponse>, StatusCode> {
    let hours = params.hours.unwrap_or(48).min(168); // Max 7 days
    
    // Fetch weather data
    let onecall_result = state.weather_client.get_onecall(params.lat, params.lon).await;
    let forecast3h_result = state.weather_client.get_forecast3h(params.lat, params.lon).await;
    
    let onecall = onecall_result.ok();
    let forecast3h = forecast3h_result.ok();
    
    if onecall.is_none() && forecast3h.is_none() {
        tracing::error!("Failed to fetch any weather data");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let merged_data = merge_weather_data(
        onecall.as_ref(),
        forecast3h.as_ref(),
        7 * 3600 // Default timezone offset
    );
    
    let hourly_data: Vec<HourlyData> = merged_data
        .into_iter()
        .take(hours as usize)
        .collect();
        
    Ok(Json(ForecastResponse {
        location: LocationInfo {
            lat: params.lat,
            lon: params.lon,
            name: None,
            country: None,
        },
        hourly_data,
        generated_at: chrono::Utc::now(),
    }))
}

pub async fn get_drying_windows(
    State(state): State<AppState>,
    Query(params): Query<DryingWindowsQuery>,
) -> Result<Json<DryingWindowsResponse>, StatusCode> {
    let window_hours = params.window_hours.unwrap_or(3).min(12);
    let max_windows = params.max_windows.unwrap_or(10).min(20);
    
    // Get weather data
    let onecall_result = state.weather_client.get_onecall(params.lat, params.lon).await;
    let forecast3h_result = state.weather_client.get_forecast3h(params.lat, params.lon).await;
    
    let onecall = onecall_result.ok();
    let forecast3h = forecast3h_result.ok();
    
    if onecall.is_none() && forecast3h.is_none() {
        tracing::error!("Failed to fetch any weather data");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let hourly_data = merge_weather_data(
        onecall.as_ref(),
        forecast3h.as_ref(),
        7 * 3600 // Default timezone offset
    );
    
    // Group into windows
    let windows = group_into_windows(&hourly_data, window_hours as i32);
    
    // Calculate scores and create response
    let mut drying_windows: Vec<DryingWindow> = windows
        .into_iter()
        .map(|window| {
            let features = WeatherFeatures {
                temp_c: window.weather.temp_c,
                rh: window.weather.rh,
                wind_ms: window.weather.wind_ms,
                cloud: window.weather.cloud,
                rain_p: 0.0, // Rain probability placeholder
                rain_mm: window.weather.rain_mm,
            };
            
            let score = calculate_drying_score(&features, &Default::default());
            
            let conditions = if window.weather.rain_mm > 0.1 {
                "Rainy".to_string()
            } else if window.weather.cloud > 80.0 {
                "Cloudy".to_string()
            } else if window.weather.cloud < 30.0 {
                "Sunny".to_string()
            } else {
                "Partly Cloudy".to_string()
            };
            
            let recommendation = if score.score > 0.8 {
                "Excellent drying conditions!".to_string()
            } else if score.score > 0.6 {
                "Good drying conditions".to_string()
            } else if score.score > 0.4 {
                "Fair drying conditions".to_string()
            } else {
                "Poor drying conditions".to_string()
            };
            
            DryingWindow {
                id: format!("window_{}_{}", window.start_time.timestamp(), window_hours),
                start_time: window.start_time.into(),
                end_time: window.end_time.into(),
                duration_hours: window_hours,
                score,
                weather_summary: WeatherSummary {
                    avg_temp_c: window.weather.temp_c,
                    avg_humidity: window.weather.rh,
                    avg_wind_ms: window.weather.wind_ms,
                    total_rain_mm: window.weather.rain_mm,
                    conditions,
                },
                recommendation,
            }
        })
        .collect();
    
    // Sort by score (best first) and limit
    drying_windows.sort_by(|a, b| b.score.score.partial_cmp(&a.score.score).unwrap());
    drying_windows.truncate(max_windows as usize);
    
    Ok(Json(DryingWindowsResponse {
        location: LocationInfo {
            lat: params.lat,
            lon: params.lon,
            name: None,
            country: None,
        },
        windows: drying_windows,
        generated_at: chrono::Utc::now(),
    }))
}

pub async fn get_recommendations(
    State(state): State<AppState>,
    Query(params): Query<RecommendationQuery>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    let window_hours = params.window_hours.unwrap_or(3);
    
    // Get user preferences if user_id provided
    let _user_prefs = if let Some(user_id) = params.user_id {
        state.database.get_user_preferences(user_id).await.ok()
    } else {
        None
    };
    
    // Get drying windows (reuse the logic)
    let windows_query = DryingWindowsQuery {
        lat: params.lat,
        lon: params.lon,
        window_hours: Some(window_hours),
        max_windows: Some(3), // Top 3 for recommendations
    };
    
    let windows_response = get_drying_windows(State(state.clone()), Query(windows_query)).await?;
    let windows_data = windows_response.0;
    
    // Generate AI explanation for the best window
    let ai_explanation = if let Some(best_window) = windows_data.windows.first() {
        let weather_features = WeatherFeatures {
            temp_c: best_window.weather_summary.avg_temp_c,
            rh: best_window.weather_summary.avg_humidity,
            wind_ms: best_window.weather_summary.avg_wind_ms,
            cloud: 50.0, // Default cloud coverage
            rain_p: if best_window.weather_summary.total_rain_mm > 0.0 { 0.8 } else { 0.0 },
            rain_mm: best_window.weather_summary.total_rain_mm,
        };
        
        let window_data = vec![(
            best_window.start_time.to_string(),
            best_window.score.clone(),
            weather_features,
        )];
        
        state.ai_client
            .explain_recommendation(
                &window_data,
                None,
            )
            .await
            .ok()
    } else {
        None
    };
    
    // Generate general tips
    let tips = if let Some(best_window) = windows_data.windows.first() {
        let weather_features = WeatherFeatures {
            temp_c: best_window.weather_summary.avg_temp_c,
            rh: best_window.weather_summary.avg_humidity,
            wind_ms: best_window.weather_summary.avg_wind_ms,
            cloud: 50.0, // Default cloud coverage
            rain_p: if best_window.weather_summary.total_rain_mm > 0.0 { 0.8 } else { 0.0 },
            rain_mm: best_window.weather_summary.total_rain_mm,
        };
        
        state.ai_client
            .generate_drying_tips(&weather_features, &best_window.score)
            .await
            .unwrap_or_else(|_| "Check weather conditions before hanging clothes. Avoid drying during rain or high humidity. Wind helps with faster drying.".to_string())
            .split(". ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        vec![
            "Check weather conditions before hanging clothes".to_string(),
            "Avoid drying during rain or high humidity".to_string(),
            "Wind helps with faster drying".to_string(),
        ]
    };
    
    Ok(Json(RecommendationResponse {
        location: windows_data.location,
        best_windows: windows_data.windows,
        ai_explanation,
        tips,
        generated_at: chrono::Utc::now(),
    }))
}

pub async fn submit_feedback(
    State(state): State<AppState>,
    Json(request): Json<FeedbackRequest>,
) -> Result<Json<FeedbackResponse>, StatusCode> {
    // Create feedback record
    let create_feedback = CreateFeedback {
        user_id: request.user_id,
        window_id: request.window_id,
        feedback_text: request.feedback_text.clone(),
        satisfaction_rating: request.satisfaction_rating,
        drying_result: request.drying_result,
        weather_temp_c: request.weather_conditions.as_ref().and_then(|w| w.temp_c),
        weather_humidity: request.weather_conditions.as_ref().and_then(|w| w.humidity),
        weather_wind_ms: request.weather_conditions.as_ref().and_then(|w| w.wind_ms),
        weather_rain_mm: request.weather_conditions.as_ref().and_then(|w| w.rain_mm),
        predicted_score: request.predicted_score,
        actual_outcome: request.actual_outcome,
    };
    
    match state.database.create_feedback(create_feedback).await {
        Ok(feedback_record) => {
            // Analyze feedback with AI
            let weather_features = if let Some(weather) = &request.weather_conditions {
                WeatherFeatures {
                    temp_c: weather.temp_c.unwrap_or(20.0),
                    rh: weather.humidity.unwrap_or(50.0),
                    wind_ms: weather.wind_ms.unwrap_or(2.0),
                    cloud: 50.0, // Default cloud coverage
                    rain_p: if weather.rain_mm.unwrap_or(0.0) > 0.0 { 0.8 } else { 0.0 },
                    rain_mm: weather.rain_mm.unwrap_or(0.0),
                }
            } else {
                WeatherFeatures {
                    temp_c: 20.0,
                    rh: 50.0,
                    wind_ms: 2.0,
                    cloud: 50.0,
                    rain_p: 0.0,
                    rain_mm: 0.0,
                }
            };
            
            let analysis = state.ai_client
                .analyze_feedback(&request.feedback_text, &weather_features)
                .await
                .ok();
            
            Ok(Json(FeedbackResponse {
                id: feedback_record.id,
                analysis,
                message: "Feedback submitted successfully".to_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to save feedback: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPreferences>, StatusCode> {
    match state.database.get_user_preferences(user_id).await {
        Ok(prefs) => Ok(Json(prefs)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_user_preferences(
    State(state): State<AppState>,
    Json(request): Json<CreateUserPreferences>,
) -> Result<Json<UserPreferences>, StatusCode> {
    match state.database.create_user_preferences(request).await {
        Ok(prefs) => Ok(Json(prefs)),
        Err(e) => {
            tracing::error!("Failed to create user preferences: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<CreateUserPreferences>,
) -> Result<Json<UserPreferences>, StatusCode> {
    match state.database.update_user_preferences(user_id, request).await {
        Ok(prefs) => Ok(Json(prefs)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn generate_recommendation_with_retry(
    ai_client: &AiClient,
    weather_features: &WeatherFeatures,
    max_retries: u32,
) -> Result<String, crate::ai::AiError> {
    let mut retries = 0;
    let mut delay = std::time::Duration::from_millis(100);
    
    loop {
        match ai_client.generate_laundry_recommendation(weather_features).await {
            Ok(recommendation) => return Ok(recommendation),
            Err(crate::ai::AiError::RateLimited) if retries < max_retries => {
                tracing::warn!("Rate limited, retrying in {:?} (attempt {})", delay, retries + 1);
                tokio::time::sleep(delay).await;
                retries += 1;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

pub async fn get_ai_recommendation(
    State(state): State<AppState>,
    Query(query): Query<AiRecommendationQuery>,
) -> Result<Json<AiRecommendationResponse>, StatusCode> {
    // Fetch weather data
    let onecall_result = state.weather_client.get_onecall(query.lat, query.lon).await;
    let forecast3h_result = state.weather_client.get_forecast3h(query.lat, query.lon).await;
    
    let onecall = onecall_result.ok();
    let forecast3h = forecast3h_result.ok();
    
    if onecall.is_none() && forecast3h.is_none() {
        tracing::error!("Failed to fetch any weather data for AI recommendation");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let merged_data = merge_weather_data(
        onecall.as_ref(),
        forecast3h.as_ref(),
        7 * 3600 // Default timezone offset
    );
    
    // Get current weather from the first hour of merged data
    let current_weather = match merged_data.first() {
        Some(hourly) => hourly,
        None => {
            tracing::error!("No current weather data available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let weather_features = WeatherFeatures {
        temp_c: current_weather.temp_c,
        rh: current_weather.rh,
        wind_ms: current_weather.wind_ms,
        cloud: current_weather.cloud,
        rain_p: current_weather.rain_p,
        rain_mm: current_weather.rain_mm,
    };

    // Generate AI recommendation with retry logic
    match generate_recommendation_with_retry(&state.ai_client, &weather_features, 3).await {
        Ok(recommendation) => Ok(Json(AiRecommendationResponse {
            recommendation,
            generated_at: chrono::Utc::now(),
        })),
        Err(e) => {
            tracing::error!("AI recommendation failed after retries: {}", e);
            match e {
                crate::ai::AiError::RateLimited => {
                    Err(StatusCode::TOO_MANY_REQUESTS)
                }
                _ => {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}

pub async fn explain_recommendation(
    State(state): State<AppState>,
    Json(request): Json<ExplainRequest>,
) -> Result<Json<ExplainResponse>, StatusCode> {
    let weather_features = WeatherFeatures {
        temp_c: request.window_data.weather.temp_c,
        rh: request.window_data.weather.rh,
        wind_ms: request.window_data.weather.wind_ms,
        cloud: request.window_data.weather.cloud,
        rain_p: request.window_data.weather.rain_p,
        rain_mm: request.window_data.weather.rain_mm,
    };
    
    let window_data = vec![(
        request.window_data.start_time.to_string(),
        request.score.clone(),
        weather_features,
    )];
    
    let user_prefs = request.user_preferences.as_ref().map(|prefs| {
        format!("Drying hours: {:?}, Min temp: {:?}, Max humidity: {:?}, Avoid rain: {:?}",
            prefs.preferred_drying_hours, prefs.min_temperature, prefs.max_humidity, prefs.avoid_rain_probability)
    });
    
    match state.ai_client.explain_recommendation(
        &window_data,
        user_prefs.as_deref(),
    ).await {
        Ok(explanation) => {
            let factors = vec![
                format!("Temperature: {:.1}°C", request.window_data.weather.temp_c),
                format!("Humidity: {:.1}%", request.window_data.weather.rh),
                format!("Wind: {:.1} m/s", request.window_data.weather.wind_ms),
                format!("Rain: {:.1} mm", request.window_data.weather.rain_mm),
                format!("Drying Score: {:.2}", request.score.score),
            ];
            
            let tips = vec![
                "Hang clothes in well-ventilated areas".to_string(),
                "Avoid direct sunlight for delicate fabrics".to_string(),
                "Shake out clothes before hanging".to_string(),
            ];
            
            Ok(Json(ExplainResponse {
                explanation,
                factors,
                tips,
            }))
        }
        Err(e) => {
            tracing::error!("AI explanation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create the router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/geocode", get(geocode))
        .route("/forecast", get(get_forecast))
        .route("/drying-windows", get(get_drying_windows))
        .route("/recommendations", get(get_recommendations))
        .route("/ai-recommendation", get(get_ai_recommendation))
        .route("/feedback", post(submit_feedback))
        .route("/preferences/:user_id", get(get_user_preferences))
        .route("/preferences/:user_id", post(update_user_preferences))
        .route("/preferences", post(create_user_preferences))
        .route("/explain", post(explain_recommendation))
        .with_state(state)
}