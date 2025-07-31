use crate::config::Config;
use crate::forecast::types::HourlyData;
use crate::scoring::{DryingScore, WeatherFeatures};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Rate limited")]
    RateLimited,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct AiClient {
    client: Client,
    config: Config,
}

impl AiClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .user_agent("LaundryDayOptimizer/1.0")
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn explain_recommendation(
        &self,
        window_data: &[(String, DryingScore, WeatherFeatures)],
        user_preferences: Option<&str>,
    ) -> Result<String, AiError> {
        let prompt = self.build_explanation_prompt(window_data, user_preferences);
        self.chat_completion(&prompt).await
    }

    pub async fn generate_drying_tips(
        &self,
        weather: &WeatherFeatures,
        _score: &DryingScore,
    ) -> Result<String, AiError> {
        let prompt = self.build_tips_prompt(weather, _score);
        self.chat_completion(&prompt).await
    }

    pub async fn analyze_feedback(
        &self,
        feedback_text: &str,
        weather_context: &WeatherFeatures,
    ) -> Result<FeedbackAnalysis, AiError> {
        let prompt = self.build_feedback_analysis_prompt(feedback_text, weather_context);
        let response = self.chat_completion(&prompt).await?;
        
        // Parse the structured response
        self.parse_feedback_analysis(&response)
    }

    pub async fn generate_laundry_recommendation(
        &self,
        weather: &WeatherFeatures,
    ) -> Result<String, AiError> {
        let prompt = self.build_laundry_recommendation_prompt(weather);
        self.chat_completion(&prompt).await
    }

    async fn chat_completion(&self, prompt: &str) -> Result<String, AiError> {
        let request = ChatRequest {
            model: self.config.or_model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant specialized in laundry drying optimization. Provide clear, practical advice based on weather conditions.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: 500,
            temperature: 0.7,
            stream: false,
        };

        let response = self
            .client
            .post(&self.config.openrouter_base_url)
            .header("Authorization", format!("Bearer {}", self.config.openrouter_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let chat_response: ChatResponse = response.json().await?;
                if let Some(choice) = chat_response.choices.first() {
                    Ok(choice.message.content.clone())
                } else {
                    Err(AiError::ApiError("No response choices".to_string()))
                }
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(AiError::RateLimited),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(AiError::ApiError(format!("HTTP {}: {}", status, error_text)))
            }
        }
    }

    fn build_explanation_prompt(
        &self,
        window_data: &[(String, DryingScore, WeatherFeatures)],
        user_preferences: Option<&str>,
    ) -> String {
        let mut prompt = String::from(
            "Explain why these laundry drying time windows are recommended based on weather conditions:\n\n",
        );

        for (window_id, score, weather) in window_data.iter().take(3) {
            prompt.push_str(&format!(
                "Window {}: Score {:.2}\n",
                window_id, score.score
            ));
            prompt.push_str(&format!(
                "- Temperature: {:.1}°C\n",
                weather.temp_c
            ));
            prompt.push_str(&format!("- Humidity: {:.1}%\n", weather.rh));
            prompt.push_str(&format!("- Wind: {:.1} m/s\n", weather.wind_ms));
            prompt.push_str(&format!("- Cloud cover: {:.1}%\n", weather.cloud * 100.0));
            prompt.push_str(&format!("- Rain probability: {:.1}%\n", weather.rain_p * 100.0));
            if weather.rain_mm > 0.0 {
                prompt.push_str(&format!("- Expected rain: {:.1}mm\n", weather.rain_mm));
            }
            prompt.push('\n');
        }

        if let Some(prefs) = user_preferences {
            prompt.push_str(&format!("User preferences: {}\n\n", prefs));
        }

        prompt.push_str(
            "Provide a concise explanation (2-3 sentences) focusing on the key weather factors that make these windows optimal for drying clothes.",
        );

        prompt
    }

    fn build_tips_prompt(&self, weather: &WeatherFeatures, score: &DryingScore) -> String {
        format!(
            "Given these weather conditions for laundry drying:\n\
             - Temperature: {:.1}°C\n\
             - Humidity: {:.1}%\n\
             - Wind: {:.1} m/s\n\
             - Cloud cover: {:.1}%\n\
             - Rain probability: {:.1}%\n\
             - Drying score: {:.2}/10\n\n\
             Provide 2-3 practical tips for optimizing laundry drying in these conditions. \
             Focus on actionable advice like clothing placement, timing, or preparation.",
            weather.temp_c,
            weather.rh,
            weather.wind_ms,
            weather.cloud * 100.0,
            weather.rain_p * 100.0,
            score.score
        )
    }

    fn build_laundry_recommendation_prompt(&self, weather: &WeatherFeatures) -> String {
        format!(
            "Based on the current weather conditions, provide a professional laundry drying recommendation:\n\n\
             Weather Details:\n\
             - Temperature: {:.1}°C\n\
             - Humidity: {:.1}%\n\
             - Wind Speed: {:.1} m/s\n\
             - Cloud Coverage: {:.1}%\n\
             - Rain Probability: {:.1}%\n\n\
             Please provide a concise, professional recommendation (2-3 sentences) about whether it's a good time to dry clothes outside. \
             Include specific advice about drying conditions and any precautions to take. \
             Be conversational and helpful, as if advising a friend.",
            weather.temp_c,
            weather.rh,
            weather.wind_ms,
            weather.cloud * 100.0,
            weather.rain_p * 100.0
        )
    }

    fn build_feedback_analysis_prompt(
        &self,
        feedback_text: &str,
        weather_context: &WeatherFeatures,
    ) -> String {
        format!(
            "Analyze this user feedback about laundry drying results:\n\
             Feedback: \"{}\"\n\n\
             Weather context:\n\
             - Temperature: {:.1}°C\n\
             - Humidity: {:.1}%\n\
             - Wind: {:.1} m/s\n\
             - Rain: {:.1}mm\n\n\
             Respond in this exact format:\n\
             SATISFACTION: [satisfied/neutral/dissatisfied]\n\
             DRYING_RESULT: [completely_dry/mostly_dry/partially_dry/not_dry]\n\
             KEY_FACTORS: [list 1-2 main weather factors mentioned]\n\
             CONFIDENCE: [high/medium/low]",
            feedback_text,
            weather_context.temp_c,
            weather_context.rh,
            weather_context.wind_ms,
            weather_context.rain_mm
        )
    }

    fn parse_feedback_analysis(&self, response: &str) -> Result<FeedbackAnalysis, AiError> {
        let mut satisfaction = FeedbackSatisfaction::Neutral;
        let mut drying_result = DryingResult::PartiallyDry;
        let mut key_factors = Vec::new();
        let mut confidence = AnalysisConfidence::Medium;

        for line in response.lines() {
            if let Some(value) = line.strip_prefix("SATISFACTION: ") {
                satisfaction = match value.trim() {
                    "satisfied" => FeedbackSatisfaction::Satisfied,
                    "dissatisfied" => FeedbackSatisfaction::Dissatisfied,
                    _ => FeedbackSatisfaction::Neutral,
                };
            } else if let Some(value) = line.strip_prefix("DRYING_RESULT: ") {
                drying_result = match value.trim() {
                    "completely_dry" => DryingResult::CompletelyDry,
                    "mostly_dry" => DryingResult::MostlyDry,
                    "not_dry" => DryingResult::NotDry,
                    _ => DryingResult::PartiallyDry,
                };
            } else if let Some(value) = line.strip_prefix("KEY_FACTORS: ") {
                key_factors = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            } else if let Some(value) = line.strip_prefix("CONFIDENCE: ") {
                confidence = match value.trim() {
                    "high" => AnalysisConfidence::High,
                    "low" => AnalysisConfidence::Low,
                    _ => AnalysisConfidence::Medium,
                };
            }
        }

        Ok(FeedbackAnalysis {
            satisfaction,
            drying_result,
            key_factors,
            confidence,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAnalysis {
    pub satisfaction: FeedbackSatisfaction,
    pub drying_result: DryingResult,
    pub key_factors: Vec<String>,
    pub confidence: AnalysisConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackSatisfaction {
    Satisfied,
    Neutral,
    Dissatisfied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DryingResult {
    CompletelyDry,
    MostlyDry,
    PartiallyDry,
    NotDry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisConfidence {
    High,
    Medium,
    Low,
}

// Mock AI client for testing
pub struct MockAiClient;

impl MockAiClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn explain_recommendation(
        &self,
        window_data: &[(String, DryingScore, WeatherFeatures)],
        _user_preferences: Option<&str>,
    ) -> Result<String, AiError> {
        if let Some((_, score, weather)) = window_data.first() {
            Ok(format!(
                "The recommended window has optimal conditions with {}°C temperature, {}% humidity, and {} m/s wind speed, resulting in a drying score of {:.1}/10. These conditions provide good evaporation rates while minimizing rain risk.",
                weather.temp_c as i32,
                weather.rh as i32,
                weather.wind_ms as i32,
                score.score
            ))
        } else {
            Ok("No suitable drying windows found in the current forecast.".to_string())
        }
    }

    pub async fn generate_drying_tips(
        &self,
        weather: &WeatherFeatures,
        _score: &DryingScore,
    ) -> Result<String, AiError> {
        let mut tips = Vec::new();

        if weather.wind_ms > 3.0 {
            tips.push("Take advantage of the strong wind by hanging clothes in open areas.");
        } else if weather.wind_ms < 1.0 {
            tips.push("With low wind, space clothes well apart for better air circulation.");
        }

        if weather.rh > 80.0 {
            tips.push("High humidity may slow drying - consider using a covered but ventilated area.");
        }

        if weather.rain_p > 0.3 {
            tips.push("Keep an eye on the sky and be ready to bring clothes in if rain starts.");
        }

        if tips.is_empty() {
            tips.push("Conditions look good for drying - hang clothes evenly spaced for best results.");
        }

        Ok(tips.join(" "))
    }

    pub async fn analyze_feedback(
        &self,
        feedback_text: &str,
        _weather_context: &WeatherFeatures,
    ) -> Result<FeedbackAnalysis, AiError> {
        let feedback_lower = feedback_text.to_lowercase();
        
        let satisfaction = if feedback_lower.contains("good") || feedback_lower.contains("great") || feedback_lower.contains("perfect") {
            FeedbackSatisfaction::Satisfied
        } else if feedback_lower.contains("bad") || feedback_lower.contains("terrible") || feedback_lower.contains("awful") {
            FeedbackSatisfaction::Dissatisfied
        } else {
            FeedbackSatisfaction::Neutral
        };

        let drying_result = if feedback_lower.contains("completely dry") || feedback_lower.contains("fully dry") {
            DryingResult::CompletelyDry
        } else if feedback_lower.contains("mostly dry") {
            DryingResult::MostlyDry
        } else if feedback_lower.contains("still wet") || feedback_lower.contains("not dry") {
            DryingResult::NotDry
        } else {
            DryingResult::PartiallyDry
        };

        Ok(FeedbackAnalysis {
            satisfaction,
            drying_result,
            key_factors: vec!["temperature".to_string(), "humidity".to_string()],
            confidence: AnalysisConfidence::Medium,
        })
    }
}