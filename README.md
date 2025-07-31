# Laundry Optimizer 🌤️👕

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next-black?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)](https://reactjs.org/)
[![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
[![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sqlite.org/)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![OpenWeather](https://img.shields.io/badge/OpenWeather-API-orange?style=flat-square&logo=openweathermap)](https://openweathermap.org/)
[![OpenRouter](https://img.shields.io/badge/OpenRouter-AI-blue?style=flat-square)](https://openrouter.ai/)
[![DeepSeek](https://img.shields.io/badge/DeepSeek-Chat-purple?style=flat-square)](https://www.deepseek.com/)

An intelligent laundry drying recommendation system that uses weather data, machine learning, and AI to suggest the optimal 3-hour windows for air-drying clothes over the next 7 days.

## 🌟 Features

### Smart Weather Analysis
- **Real-time Weather Data**: Integrates with OpenWeather API for accurate forecasts
- **7-Day Predictions**: Analyzes weather patterns up to 7 days ahead
- **Multi-source Data**: Combines One Call 3.0 API and 5-day/3-hour forecasts
- **Location Services**: Automatic location detection with manual city search fallback

### AI-Powered Recommendations
- **Machine Learning Scoring**: Advanced drying score calculation using weather features
- **Natural Language Explanations**: AI-generated rationales for recommendations
- **Personalized Learning**: Adapts to user feedback with online learning algorithms
- **Smart Preferences**: Natural language preference parsing

### Advanced Scoring System
- **Multi-factor Analysis**: Temperature, humidity, wind speed, cloud cover, rain probability, and VPD (Vapor Pressure Deficit)
- **Safety Vetoes**: Automatically excludes unsafe drying conditions
- **Soft Penalties**: Adjusts scores for suboptimal conditions
- **User Feedback Integration**: Learns from 👍/👎 feedback to improve recommendations

### Modern Tech Stack
- **Frontend**: Next.js 15 with React 19, TypeScript, and Tailwind CSS
- **Backend**: Rust with Axum framework for high-performance API
- **Database**: SQLite with SQLx for data persistence
- **AI Integration**: OpenRouter API with DeepSeek model
- **Caching**: Intelligent caching with Moka for optimal performance

## 🚀 Quick Start

### Prerequisites

- **Node.js** 18+ and npm/pnpm
- **Rust** 1.70+ ([Install from rustup.rs](https://rustup.rs/))
- **OpenWeather API Key** ([Get free key](https://openweathermap.org/api))
- **OpenRouter API Key** ([Sign up](https://openrouter.ai/))

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd laundry-optimizer
   ```

2. **Set up the backend**:
   ```bash
   cd server
   
   # Install Rust dependencies
   cargo build
   
   # Configure environment variables
   cp .env.example .env
   # Edit .env with your API keys
   ```

3. **Set up the frontend**:
   ```bash
   cd ../client
   
   # Install Node.js dependencies
   npm install
   # or
   pnpm install
   ```

### Configuration

Edit `server/.env` with your API credentials:

```env
# Required API Keys
OPENWEATHER_API_KEY=your_openweather_api_key_here
OPENROUTER_API_KEY=your_openrouter_api_key_here

# Database (SQLite by default)
DATABASE_URL=sqlite:./laundry_optimizer.db

# Server Configuration
SERVER_PORT=8080
SERVER_HOST=0.0.0.0
```

### Running the Application

1. **Start the backend server**:
   ```bash
   cd server
   cargo run
   ```
   The API will be available at `http://localhost:8080`
   
   API documentation: `http://localhost:8080/swagger-ui/`

2. **Start the frontend** (in a new terminal):
   ```bash
   cd client
   npm run dev
   # or
   pnpm dev
   ```
   The web app will be available at `http://localhost:3000`

## 📱 Usage

### Home Screen
- View **Top 3** recommended drying windows
- Each recommendation shows:
  - Drying score and time range
  - AI-generated explanation
  - Helpful tips
  - Feedback buttons (👍/👎)

### Timeline View
- Interactive 7-day heatmap of drying conditions
- Color-coded 3-hour time slots
- Tap any slot for detailed weather information
- "Explain" button for AI analysis

### Settings
- **Location**: Auto-detect or manual city search
- **Preferences**: Natural language preference input
- **Personalization**: System learns from your feedback

## 🧠 How It Works

### Drying Score Algorithm

The system calculates a comprehensive drying score using:

- **Temperature Factor**: Optimal range 15-30°C
- **Humidity Factor**: Lower humidity = better drying
- **Wind Factor**: Gentle breeze improves evaporation
- **Cloud Cover**: Sunny conditions preferred
- **Rain Probability**: Avoids rainy periods
- **VPD (Vapor Pressure Deficit)**: Scientific measure of air's drying capacity

### Safety Features

- **Hard Vetoes**: Automatically excludes windows with >50% rain probability or >0.2mm expected rainfall
- **Soft Penalties**: Reduces scores for suboptimal conditions (low temperature, no wind)
- **Real-time Updates**: Continuously monitors weather changes

### Machine Learning

- **Online Learning**: Adapts to user feedback using logistic regression
- **Personalized Weights**: Each user's preferences influence future recommendations
- **Bounded Updates**: Prevents overfitting with regularization

## 🛠️ Development

### Backend Architecture

```
server/src/
├── main.rs          # Application entry point
├── routes.rs        # API endpoints
├── ai.rs           # AI integration (OpenRouter/DeepSeek)
├── scoring.rs      # Drying score calculation
├── forecast/       # Weather data processing
├── database.rs     # Data persistence
└── config.rs       # Configuration management
```

### Frontend Structure

```
client/
├── app/            # Next.js app directory
├── components/     # Reusable UI components
├── hooks/          # Custom React hooks
├── lib/            # Utility functions
└── styles/         # Global styles
```

### API Endpoints

- `GET /api/geocode` - Location search and coordinates
- `GET /api/forecast` - Weather forecast data
- `GET /api/recommendations` - Top 3 drying windows
- `POST /api/feedback` - User feedback submission
- `POST /api/ai/explain` - AI explanation generation
- `GET /swagger-ui/` - Interactive API documentation

## 🧪 Testing

```bash
# Backend tests
cd server
cargo test

# Frontend tests
cd client
npm test
```

## 📊 Performance

- **Caching**: Weather data cached for optimal API usage
- **Efficient Scoring**: Vectorized calculations for fast processing
- **Rate Limiting**: Built-in protection against API abuse
- **Async Processing**: Non-blocking operations throughout

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [OpenWeather](https://openweathermap.org/) for weather data
- [OpenRouter](https://openrouter.ai/) for AI capabilities
- [DeepSeek](https://www.deepseek.com/) for the AI model
- The Rust and React communities for excellent tooling

---

**Made with ❤️ for smarter laundry days**