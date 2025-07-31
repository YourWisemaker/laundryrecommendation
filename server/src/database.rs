use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub preferred_drying_hours: Option<i32>,
    pub min_temperature: Option<f64>,
    pub max_humidity: Option<f64>,
    pub avoid_rain_probability: Option<f64>,
    pub location_lat: Option<f64>,
    pub location_lon: Option<f64>,
    pub location_name: Option<String>,
    pub timezone: Option<String>,
    pub notification_preferences: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeedbackRecord {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub window_id: String,
    pub feedback_text: String,
    pub satisfaction_rating: Option<i32>, // 1-5 scale
    pub drying_result: Option<String>,
    pub weather_temp_c: Option<f64>,
    pub weather_humidity: Option<f64>,
    pub weather_wind_ms: Option<f64>,
    pub weather_rain_mm: Option<f64>,
    pub predicted_score: Option<f64>,
    pub actual_outcome: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserPreferences {
    pub preferred_drying_hours: Option<i32>,
    pub min_temperature: Option<f64>,
    pub max_humidity: Option<f64>,
    pub avoid_rain_probability: Option<f64>,
    pub location_lat: Option<f64>,
    pub location_lon: Option<f64>,
    pub location_name: Option<String>,
    pub timezone: Option<String>,
    pub notification_preferences: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFeedback {
    pub user_id: Option<Uuid>,
    pub window_id: String,
    pub feedback_text: String,
    pub satisfaction_rating: Option<i32>,
    pub drying_result: Option<String>,
    pub weather_temp_c: Option<f64>,
    pub weather_humidity: Option<f64>,
    pub weather_wind_ms: Option<f64>,
    pub weather_rain_mm: Option<f64>,
    pub predicted_score: Option<f64>,
    pub actual_outcome: Option<String>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_tables(&self) -> Result<(), DatabaseError> {
        // Create user_preferences table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_preferences (
                user_id TEXT PRIMARY KEY,
                preferred_drying_hours INTEGER,
                min_temperature REAL,
                max_humidity REAL,
                avoid_rain_probability REAL,
                location_lat REAL,
                location_lon REAL,
                location_name TEXT,
                timezone TEXT,
                notification_preferences TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create feedback table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS feedback (
                id TEXT PRIMARY KEY,
                user_id TEXT REFERENCES user_preferences(user_id),
                window_id TEXT NOT NULL,
                feedback_text TEXT NOT NULL,
                satisfaction_rating INTEGER CHECK (satisfaction_rating >= 1 AND satisfaction_rating <= 5),
                drying_result TEXT,
                weather_temp_c REAL,
                weather_humidity REAL,
                weather_wind_ms REAL,
                weather_rain_mm REAL,
                predicted_score REAL,
                actual_outcome TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_feedback_user_id ON feedback(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_feedback_created_at ON feedback(created_at)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // User Preferences CRUD
    pub async fn create_user_preferences(
        &self,
        prefs: CreateUserPreferences,
    ) -> Result<UserPreferences, DatabaseError> {
        let user_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let result = sqlx::query_as::<_, UserPreferences>(
            r#"
            INSERT INTO user_preferences (
                user_id, preferred_drying_hours, min_temperature, max_humidity,
                avoid_rain_probability, location_lat, location_lon, location_name,
                timezone, notification_preferences, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(prefs.preferred_drying_hours)
        .bind(prefs.min_temperature)
        .bind(prefs.max_humidity)
        .bind(prefs.avoid_rain_probability)
        .bind(prefs.location_lat)
        .bind(prefs.location_lon)
        .bind(prefs.location_name)
        .bind(prefs.timezone)
        .bind(prefs.notification_preferences)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<UserPreferences, DatabaseError> {
        let result = sqlx::query_as::<_, UserPreferences>(
            "SELECT * FROM user_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::UserNotFound)?;

        Ok(result)
    }

    pub async fn update_user_preferences(
        &self,
        user_id: Uuid,
        prefs: CreateUserPreferences,
    ) -> Result<UserPreferences, DatabaseError> {
        let now = chrono::Utc::now();

        let result = sqlx::query_as::<_, UserPreferences>(
            r#"
            UPDATE user_preferences SET
                preferred_drying_hours = COALESCE($2, preferred_drying_hours),
                min_temperature = COALESCE($3, min_temperature),
                max_humidity = COALESCE($4, max_humidity),
                avoid_rain_probability = COALESCE($5, avoid_rain_probability),
                location_lat = COALESCE($6, location_lat),
                location_lon = COALESCE($7, location_lon),
                location_name = COALESCE($8, location_name),
                timezone = COALESCE($9, timezone),
                notification_preferences = COALESCE($10, notification_preferences),
                updated_at = $11
            WHERE user_id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(prefs.preferred_drying_hours)
        .bind(prefs.min_temperature)
        .bind(prefs.max_humidity)
        .bind(prefs.avoid_rain_probability)
        .bind(prefs.location_lat)
        .bind(prefs.location_lon)
        .bind(prefs.location_name)
        .bind(prefs.timezone)
        .bind(prefs.notification_preferences)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::UserNotFound)?;

        Ok(result)
    }

    // Feedback CRUD
    pub async fn create_feedback(
        &self,
        feedback: CreateFeedback,
    ) -> Result<FeedbackRecord, DatabaseError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let result = sqlx::query_as::<_, FeedbackRecord>(
            r#"
            INSERT INTO feedback (
                id, user_id, window_id, feedback_text, satisfaction_rating,
                drying_result, weather_temp_c, weather_humidity, weather_wind_ms,
                weather_rain_mm, predicted_score, actual_outcome, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(feedback.user_id)
        .bind(feedback.window_id)
        .bind(feedback.feedback_text)
        .bind(feedback.satisfaction_rating)
        .bind(feedback.drying_result)
        .bind(feedback.weather_temp_c)
        .bind(feedback.weather_humidity)
        .bind(feedback.weather_wind_ms)
        .bind(feedback.weather_rain_mm)
        .bind(feedback.predicted_score)
        .bind(feedback.actual_outcome)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_user_feedback(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<FeedbackRecord>, DatabaseError> {
        let limit = limit.unwrap_or(50).min(100); // Cap at 100

        let results = sqlx::query_as::<_, FeedbackRecord>(
            "SELECT * FROM feedback WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    pub async fn get_recent_feedback(
        &self,
        days: i32,
        limit: Option<i64>,
    ) -> Result<Vec<FeedbackRecord>, DatabaseError> {
        let limit = limit.unwrap_or(100).min(500); // Cap at 500
        let since = chrono::Utc::now() - chrono::Duration::days(days as i64);

        let results = sqlx::query_as::<_, FeedbackRecord>(
            "SELECT * FROM feedback WHERE created_at >= $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    // Analytics queries
    pub async fn get_feedback_stats(&self) -> Result<HashMap<String, serde_json::Value>, DatabaseError> {
        let mut stats = HashMap::new();

        // Total feedback count
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM feedback")
            .fetch_one(&self.pool)
            .await?;
        stats.insert("total_feedback".to_string(), serde_json::Value::Number(total_count.into()));

        // Average satisfaction rating
        let avg_satisfaction: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(satisfaction_rating::FLOAT) FROM feedback WHERE satisfaction_rating IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;
        if let Some(avg) = avg_satisfaction {
            stats.insert("avg_satisfaction".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(avg).unwrap_or_else(|| serde_json::Number::from(0))
            ));
        }

        // Feedback by drying result
        let drying_results = sqlx::query(
            "SELECT drying_result, COUNT(*) as count FROM feedback WHERE drying_result IS NOT NULL GROUP BY drying_result"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result_counts = HashMap::new();
        for row in drying_results {
            let result: String = row.get("drying_result");
            let count: i64 = row.get("count");
            result_counts.insert(result, serde_json::Value::Number(count.into()));
        }
        stats.insert("drying_results".to_string(), serde_json::Value::Object(
            result_counts.into_iter().collect()
        ));

        Ok(stats)
    }

    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}

// Mock database for testing
pub struct MockDatabase {
    users: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, UserPreferences>>>,
    feedback: std::sync::Arc<tokio::sync::RwLock<Vec<FeedbackRecord>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            users: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            feedback: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn create_user_preferences(
        &self,
        prefs: CreateUserPreferences,
    ) -> Result<UserPreferences, DatabaseError> {
        let user_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let user_prefs = UserPreferences {
            user_id,
            preferred_drying_hours: prefs.preferred_drying_hours,
            min_temperature: prefs.min_temperature,
            max_humidity: prefs.max_humidity,
            avoid_rain_probability: prefs.avoid_rain_probability,
            location_lat: prefs.location_lat,
            location_lon: prefs.location_lon,
            location_name: prefs.location_name,
            timezone: prefs.timezone,
            notification_preferences: prefs.notification_preferences,
            created_at: now,
            updated_at: now,
        };

        self.users.write().await.insert(user_id, user_prefs.clone());
        Ok(user_prefs)
    }

    pub async fn get_user_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<UserPreferences, DatabaseError> {
        self.users
            .read()
            .await
            .get(&user_id)
            .cloned()
            .ok_or(DatabaseError::UserNotFound)
    }

    pub async fn create_feedback(
        &self,
        feedback: CreateFeedback,
    ) -> Result<FeedbackRecord, DatabaseError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let feedback_record = FeedbackRecord {
            id,
            user_id: feedback.user_id,
            window_id: feedback.window_id,
            feedback_text: feedback.feedback_text,
            satisfaction_rating: feedback.satisfaction_rating,
            drying_result: feedback.drying_result,
            weather_temp_c: feedback.weather_temp_c,
            weather_humidity: feedback.weather_humidity,
            weather_wind_ms: feedback.weather_wind_ms,
            weather_rain_mm: feedback.weather_rain_mm,
            predicted_score: feedback.predicted_score,
            actual_outcome: feedback.actual_outcome,
            created_at: now,
        };

        self.feedback.write().await.push(feedback_record.clone());
        Ok(feedback_record)
    }

    pub async fn get_user_feedback(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<FeedbackRecord>, DatabaseError> {
        let limit = limit.unwrap_or(50) as usize;
        let feedback = self.feedback.read().await;
        
        let mut user_feedback: Vec<_> = feedback
            .iter()
            .filter(|f| f.user_id == Some(user_id))
            .cloned()
            .collect();
        
        user_feedback.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        user_feedback.truncate(limit);
        
        Ok(user_feedback)
    }
}