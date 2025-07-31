# Laundry Day Optimizer - Backend Server

A Rust/Axum backend server that provides weather-based laundry drying recommendations using machine learning and AI.

## Features

- **Weather Integration**: Fetches real-time and forecast data from OpenWeather API
- **Smart Scoring**: ML-based drying score calculation with online learning
- **AI Recommendations**: Natural language explanations via OpenRouter/DeepSeek
- **Caching**: Intelligent caching for weather data and API responses
- **Database**: PostgreSQL for user preferences and feedback storage
- **API Documentation**: Auto-generated OpenAPI/Swagger documentation

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- PostgreSQL 12+ (local or cloud instance)
- OpenWeather API key (free tier available)
- OpenRouter API key (for AI features)

### Installation

1. **Clone and navigate to server directory**:
   ```bash
   cd server
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your API keys and database URL
   ```

4. **Set up database**:
   ```bash
   # Create database
   createdb laundry_optimizer
   
   # Run migrations (tables will be created automatically on first run)
   ```

5. **Run the server**:
   ```bash
   cargo run
   ```

The server will start on `http://localhost:8080`

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|----------|
| `OPENWEATHER_API_KEY` | OpenWeather API key | `your_api_key_here` |
| `OPENROUTER_API_KEY` | OpenRouter API key | `your_api_key_here` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@localhost:5432/db` |

### Optional

| Variable | Description | Default |
|----------|-------------|----------|
| `APP_TIMEZONE` | Application timezone | `UTC` |
| `SERVER_PORT` | Server port | `8080` |
| `SERVER_HOST` | Server host | `0.0.0.0` |
| `RUST_LOG` | Log level | `info` |
| `OPENROUTER_MODEL` | AI model to use | `deepseek/deepseek-chat` |

See `.env.example` for all available configuration options.

## API Endpoints

### Core Endpoints

- `GET /health` - Health check
- `GET /api/geocode` - Geocode location by name
- `GET /api/forecast` - Get weather forecast
- `GET /api/drying-windows` - Get optimal drying windows
- `GET /api/recommendations` - Get AI-powered recommendations
- `POST /api/feedback` - Submit user feedback
- `GET /api/preferences/{user_id}` - Get user preferences
- `PUT /api/preferences/{user_id}` - Update user preferences
- `POST /api/explain` - Get AI explanation for recommendations

### Documentation

- `GET /swagger-ui/` - Interactive API documentation
- `GET /api-docs/openapi.json` - OpenAPI specification

## Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── routes.rs            # API route handlers
├── scoring.rs           # Drying score calculation & ML
├── database.rs          # Database operations
├── ai.rs               # AI/OpenRouter integration
├── utils.rs            # Utility functions
└── forecast/           # Weather data module
    ├── mod.rs          # Module definition & caching
    ├── types.rs        # Data structures
    ├── openweather.rs  # OpenWeather API client
    ├── merge.rs        # Data merging logic
    └── mock.rs         # Mock client for testing
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test scoring::tests
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features
```

### Development Server

```bash
# Run with auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

## Machine Learning

The server implements online learning for drying score optimization:

- **Initial Weights**: Based on meteorological research
- **Feedback Loop**: User feedback updates model weights via SGD
- **Features**: Temperature, humidity, wind speed, cloud cover, UV index
- **Scoring**: Combines normalized features with learned weights
- **Vetoes**: Hard constraints for rain, extreme temperatures

## AI Integration

AI features powered by OpenRouter/DeepSeek:

- **Explanations**: Natural language reasoning for recommendations
- **Tips**: Contextual drying advice based on conditions
- **Feedback Analysis**: Structured analysis of user feedback

## Caching Strategy

- **Weather Data**: 30-minute TTL for forecast data
- **Geocoding**: 24-hour TTL for location lookups
- **AI Responses**: 1-hour TTL for similar queries
- **Memory Management**: LRU eviction with size limits

## Error Handling

- **Graceful Degradation**: Fallback to cached/mock data
- **Retry Logic**: Exponential backoff for API calls
- **Structured Errors**: Consistent error response format
- **Logging**: Comprehensive request/error logging

## Performance

- **Async/Await**: Non-blocking I/O throughout
- **Connection Pooling**: Database connection management
- **Rate Limiting**: Configurable per-endpoint limits
- **Compression**: Gzip response compression

## Security

- **CORS**: Configurable cross-origin policies
- **Input Validation**: Comprehensive request validation
- **SQL Injection**: Parameterized queries via SQLx
- **API Keys**: Environment-based secret management

## Deployment

### Docker

```bash
# Build image
docker build -t laundry-optimizer-server .

# Run container
docker run -p 8080:8080 --env-file .env laundry-optimizer-server
```

### Production Checklist

- [ ] Set `RUST_LOG=warn` or `error` in production
- [ ] Use connection pooling for database
- [ ] Set up proper CORS origins
- [ ] Configure rate limiting
- [ ] Set up monitoring and health checks
- [ ] Use HTTPS in production
- [ ] Rotate API keys regularly

## Monitoring

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Database health check
curl http://localhost:8080/health?check=db
```

### Metrics

The server exposes metrics via structured logging:

- Request/response times
- API call success/failure rates
- Cache hit/miss ratios
- Database query performance
- ML model accuracy metrics

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Check `DATABASE_URL` format
   - Ensure PostgreSQL is running
   - Verify credentials and database exists

2. **API Key Errors**
   - Verify OpenWeather API key is valid
   - Check OpenRouter API key and credits
   - Ensure keys are properly set in `.env`

3. **High Memory Usage**
   - Reduce cache sizes in configuration
   - Check for memory leaks in long-running processes
   - Monitor database connection pool

4. **Slow Response Times**
   - Check external API response times
   - Verify database query performance
   - Review cache hit rates

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Enable SQL query logging
RUST_LOG=sqlx=debug cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings
- Add documentation for public APIs
- Include unit tests for new functions
- Update integration tests for API changes

## License

This project is licensed under the MIT License - see the LICENSE file for details.