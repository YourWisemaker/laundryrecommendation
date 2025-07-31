use axum::Router;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use sqlx::SqlitePool;

mod ai;
mod config;
mod database;
mod forecast;
mod routes;
mod scoring;

use ai::AiClient;
use config::Config;
use database::Database;
use forecast::openweather::OpenWeatherClient;
use routes::{create_router, AppState};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "laundry_optimizer_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./laundry_optimizer.db".to_string());
    let pool = sqlx::SqlitePool::connect(&database_url).await?;
    let database = Arc::new(Database::new(pool));
    
    // Initialize database tables
    database.init_tables().await?;
    
    // Initialize weather client
    let weather_client = Arc::new(OpenWeatherClient::new(config.clone()));
    
    // Initialize AI client
    let ai_client = Arc::new(AiClient::new(config.clone()));
    
    let config = Arc::new(config);
    
    // Create application state
    let state = AppState {
        config,
        database,
        weather_client,
        ai_client,
    };

    let app = create_router(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server starting on http://0.0.0.0:8080");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}