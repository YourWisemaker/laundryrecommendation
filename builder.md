You are generating a complete, runnable mini‑app called **Laundry Day Optimizer**.

Goal
-----
A mobile app that recommends the best 3‑hour windows over the next 7 days to air‑dry clothes.

Tech
----
Frontend: **React Native (Expo)**
Backend: **Rust (Axum)**
Weather provider: **OpenWeather** (One Call 3.0 + 5‑day/3‑hour + Geocoding)
AI provider: **OpenRouter** using ONLY model **deepseek/deepseek-chat-v3-0324:free**
Timezone: Display in device local time; default to **Asia/Jakarta (UTC+7)** if unknown
Security: Never hardcode secrets; read env vars on the server only

Functional Requirements
-----------------------
1) Weather ingestion (OpenWeather):
   - One Call 3.0 (hourly+daily) `/data/3.0/onecall?lat&lon&units=metric&exclude=minutely,alerts&appid`
     Use fields: hourly[].temp (°C), hourly[].humidity (%), hourly[].wind_speed (m/s),
                 hourly[].clouds (%), hourly[].pop (0..1), hourly[].rain.1h (mm, optional),
                 timezone_offset (seconds)
   - 5‑day / 3‑hour forecast `/data/2.5/forecast?lat&lon&units=metric&appid`
     Use fields: list[].main.temp, main.humidity, wind.speed, clouds.all, pop, rain.3h (mm)
   - Geocoding `/geo/1.0/direct?q=<city>&limit=1&appid` and reverse geocode `/geo/1.0/reverse?lat&lon&limit=1&appid` (optional)
   - Normalize into unified hourly rows: { ts, temp_c, rh, wind_ms, cloud, rain_p, rain_mm }
     Merge strategy:
       • Prefer 3‑hour forecast for exact 3‑hour steps up to 120h
       • Else One Call hourly for 0–48h
       • Else synthesize from daily up to day 7
   - Daily→3‑hour synthesis (if only daily available):
       • Fill 8 bins/day with daily values, adjust diurnally:
         Daylight: +1.0°C temp, −5% RH, −10% cloud (clamped 0..1)
         Night:    −1.0°C temp, +5% RH, +10% cloud
       • Distribute daily rain mm evenly; convert pop to per‑bin rain_p

2) Windowing & scoring:
   - Group hourly timeline into **3‑hour windows** for 7 days (configurable step_hours)
   - Compute features & **Drying Score** per window; mark unsafe windows via hard veto
   - Return **Top 3** safe windows ranked by score

3) Feedback & learning:
   - Users send 👍/👎 feedback per window
   - Store feedback and apply a small online logistic update to per‑user weights (bounded)

4) AI (OpenRouter / DeepSeek):
   - **EXPLAIN**: numeric features → concise rationale + 1 short tip (strict JSON)
   - **PREFS**: free‑text → structured constraints (strict JSON)
   - **WEIGHT_TUNING**: propose small deltas to weights within bounds (strict JSON; server validates)

5) Frontend UX:
   - **Home**: Top‑3 cards: score chip, time range, 2‑line rationale, tiny “Tip”, 👍/👎
   - **Timeline**: 7‑day horizontal heatbar (3‑hour bins); tap bin → details + “Explain”
   - **Settings**: “Use device location” toggle, city search, preference text (parsed via AI)
   - **Auto current location** (Expo Location):
       • Ask foreground permission on first run
       • Use last known location, else current with timeout
       • Cache {lat, lon, ts, label?} for 10 minutes in AsyncStorage
       • If denied, fall back to manual city search via backend geocode
       • Never request background location; store only lat/lon + optional city string

Drying Score — Formula & Functions (Source of Truth)
----------------------------------------------------
Features (normalize to [0,1] unless noted):
- f_temp  = clamp((temp_c - 15)/15, 0, 1)
- f_hum   = 1 - (rh/100)^(0.7)
- f_wind  = clamp(wind_ms/6, 0, 1)
- f_cloud = 1 - clamp(cloud, 0, 1)       // cloud supplied as 0..1
- f_rain  = 1 - clamp(rain_p, 0, 1)
- VPD (kPa):
    es = 0.6108 * exp((17.27 * temp_c) / (temp_c + 237.3))
    e  = es * (rh/100)
    VPD_kPa = max(es - e, 0)
  f_vpd   = clamp(VPD_kPa/2.5, 0, 1)

Hard veto: if rain_p > 0.50 OR expected_rain_mm > 0.2 ⇒ unsafe=true and score = -1.0 (sink rank)

Linear score:
  score = w0 + w1*f_temp + w2*f_hum + w3*f_wind + w4*f_cloud + w5*f_rain + w6*f_vpd

Default weights:
  w0=0, w1=0.25, w2=0.25, w3=0.20, w4=0.10, w5=0.15, w6=0.25

Soft penalties (after linear):
  if temp_c < 18°C ⇒ −0.15
  if wind_ms < 1   ⇒ −0.10

Online update (binary feedback y∈{0,1}) – logistic step:
  z = w·x; p = σ(z) = 1/(1+e^-z)
  w := w - η * (p - y) * x  + 2λw     (η≈0.05, λ≈1e-4)
Where x = [1, f_temp, f_hum, f_wind, f_cloud, f_rain, f_vpd]

Backend (Rust, Axum) — Deliverables
-----------------------------------
Crates: axum, tokio, tower-http, serde, serde_json, thiserror, reqwest, chrono, time, tzdb,
sqlx (SQLite by default), uuid, moka (cache), tracing, tracing-subscriber, utoipa (+swagger-ui), anyhow

Project layout:
/server
  src/main.rs
  src/config.rs                      // env (OpenRouter, OpenWeather, toggles)
  src/forecast/{mod.rs, openweather.rs, mock.rs, types.rs, merge.rs}
  src/scoring.rs                     // VPD, features, score, SGD update
  src/ai/{client.rs, system_prompt.rs, tasks.rs}  // OpenRouter client + prompts
  src/db/{mod.rs, schema.sql, models.rs}
  src/routes/{windows.rs, recommend.rs, feedback.rs, prefs.rs, explain.rs, geocode.rs, health.rs}
  src/util/{time.rs, id.rs}
  tests/ (unit: scoring, sgd, veto; integration: endpoints)
.server/.env.example
.server/README.md

Environment (.env.example):
# OpenRouter
OPENROUTER_API_KEY=YOUR_OPENROUTER_KEY
OR_MODEL=deepseek/deepseek-chat-v3-0324:free

# OpenWeather
OPENWEATHER_API_KEY=YOUR_OPENWEATHER_KEY
OPENWEATHER_BASE_URL=https://api.openweathermap.org
OPENWEATHER_ONECALL_PATH=/data/3.0/onecall
OPENWEATHER_FORECAST3H_PATH=/data/2.5/forecast
OPENWEATHER_GEOCODE_DIRECT_PATH=/geo/1.0/direct
OPENWEATHER_GEOCODE_REVERSE_PATH=/geo/1.0/reverse

# App
APP_TIMEZONE=Asia/Jakarta

OpenWeather client:
- Units: metric
- Headers: User-Agent: LaundryDayOptimizer/1.0
- 429 handling: exponential backoff with jitter
- Caching (Moka): OneCall 30 min, Forecast3h 30 min, Geocode 24 h
- Merge hourly + 3‑hour + daily per strategy above

OpenWeather → Internal field mapping:
OneCall hourly → internal hour:
  temp_c=hourly[i].temp; rh=hourly[i].humidity; wind_ms=hourly[i].wind_speed;
  cloud=hourly[i].clouds/100.0; rain_p=hourly[i].pop; rain_mm=hourly[i].rain?.["1h"]||0
  ts = (hourly[i].dt + timezone_offset) seconds → ISO with offset
Forecast3h list → internal hour (replicate each 3‑hour step to 3 hours):
  temp_c=list[i].main.temp; rh=list[i].main.humidity; wind_ms=list[i].wind.speed;
  cloud=list[i].clouds.all/100.0; rain_p=list[i].pop; rain_mm=list[i].rain?.["3h"]||0
Daily → synthesized (8 bins/day):
  base from daily.temp.day, humidity, wind_speed, clouds/100, pop, rain (mm/day)
  distribute rain_mm evenly; apply daylight/night bias

AI (OpenRouter / DeepSeek) — Prompts & Calls
--------------------------------------------
System prompt (create `src/ai/system_prompt.rs` EXACT TEXT):
pub const SYSTEM_PROMPT: &str = r#"
You are “Laundry Day Optimizer AI,” a concise, mobile-first assistant embedded in a
Rust + React Native app. You never invent weather data; you only interpret numeric
features the backend provides. Your mission:
1) Explain drying quality for a 3‑hour window in 1–2 sentences + one actionable tip.
2) Parse free‑text user preferences into a strict JSON schema (SCHEMA_PREFS).
3) Suggest small, bounded adjustments to drying weights based on recent feedback.
DRYING FORMULA and rules:
f_temp=clamp((t-15)/15,0,1); f_hum=1-(rh/100)^0.7; f_wind=clamp(w/6,0,1);
f_cloud=1-clamp(cloud,0,1); f_rain=1-clamp(rain_p,0,1);
es=0.6108*exp((17.27*t)/(t+237.3)); VPD_kPa=max(es - es*(rh/100), 0);
f_vpd=clamp(VPD_kPa/2.5,0,1).
Hard veto: rain_p>0.50 OR expected_rain_mm>0.2 ⇒ unsafe.
score = w0 + w1*f_temp + w2*f_hum + w3*f_wind + w4*f_cloud + w5*f_rain + w6*f_vpd.
Default weights: w0=0,w1=0.25,w2=0.25,w3=0.20,w4=0.10,w5=0.15,w6=0.25.
Penalties: temp_c<18 ⇒ −0.15; wind_ms<1 ⇒ −0.10.
OUTPUT (STRICT JSON ONLY):
SCHEMA_EXPLAIN: {"rationale":string<=2 sentences,"tip":string<=1 short sentence,
                 "verdict":"great|good|ok|avoid","reasons":["≤3 keywords"]}
SCHEMA_PREFS:   {"avoid_hours":[0..23],"min_temp_c":number|null,"max_rain_p":number|null,
                 "prioritize":["wind","sun","warmth","low_humidity"]}
SCHEMA_WEIGHTS: {"delta":{"w1":num,"w2":num,"w3":num,"w4":num,"w5":num,"w6":num},
                 "justification":"≤2 sentences","bounds_respected":true}
STYLE: concise, numeric units (°C, m/s, %). If unsafe=true, mention rain explicitly.
TASK SWITCH: user message includes {"task":"EXPLAIN"|"PREFS"|"WEIGHT_TUNING"}; return only that schema.
"#;

User payload builders:
- EXPLAIN:
  {"task":"EXPLAIN","window":{"start_iso":"...","end_iso":"...","unsafe":false},
   "raw":{"temp_c":x,"rh":x,"wind_ms":x,"rain_p":x,"cloud":x,"vpd_kpa":x},
   "features":{"f_temp":x,"f_hum":x,"f_wind":x,"f_cloud":x,"f_rain":x,"f_vpd":x},
   "score":x,"weights":{"w0":0,"w1":0.25,"w2":0.25,"w3":0.20,"w4":0.10,"w5":0.15,"w6":0.25}}
- PREFS:
  {"task":"PREFS","text":"<user free text>"}
- WEIGHT_TUNING:
  {"task":"WEIGHT_TUNING","summary":{...},"current_weights":{...},"observed":{...}}

OpenRouter client:
- Env: OPENROUTER_API_KEY, OR_MODEL (default "deepseek/deepseek-chat-v3-0324:free")
- POST https://openrouter.ai/api/v1/chat/completions
- Headers: Authorization: Bearer <env>, HTTP-Referer: https://laundry.example, X-Title: Laundry Day Optimizer
- Params: temperature=0.3, max_tokens=200, response_format { "type": "json_object" }
- Parse choices[0].message.content strictly as JSON; on invalid JSON respond 502 with error

Endpoints (JSON; ISO timestamps with offset)
--------------------------------------------
- GET  /health -> {status:"ok"}
- POST /geocode { "q": "Jakarta" } -> { lat, lon, name, country } (direct or reverse based on input)
- POST /forecast/refresh {lat,lon} -> caches OneCall + (optional) Forecast3h
- GET  /drying/windows?lat&lon&days=7&step_hours=3 -> [{window}]
- GET  /recommendations/top?lat&lon&limit=3 -> top windows (unsafe filtered)
- POST /feedback {window_id, rating(0|1), note?} -> {ok:true}
- POST /prefs {text} -> parsed prefs JSON; persist per user
- GET  /explain?window_id=... -> SCHEMA_EXPLAIN JSON via OpenRouter
- GET  /docs -> Swagger UI

Frontend (React Native, Expo)
-----------------------------
Project layout:
/app
  App.tsx (providers, theme)
  src/api/client.ts (EXPO_PUBLIC_API)
  src/context/LocationContext.tsx
  src/hooks/{useTop.ts,useExplain.ts,useFeedback.ts,useDeviceLocation.ts}
  src/screens/{Home.tsx, Timeline.tsx, Settings.tsx, LocationGate.tsx}
  src/components/{SlotCard.tsx, ScoreChip.tsx, Heatbar.tsx}
  src/theme/colors.ts (tailwind-like tokens)
  app.json, .env.example

Auto current location (Expo):
- Dependencies: expo-location, @tanstack/react-query, @react-native-async-storage/async-storage
- app.json plugin & permissions:
  iOS: NSLocationWhenInUseUsageDescription = "We use your location to find the best outdoor drying times for your area."
  Android permissions: ACCESS_COARSE_LOCATION, ACCESS_FINE_LOCATION
- Hook `useDeviceLocation`:
  • Foreground permission request
  • Try getLastKnownPositionAsync(); else getCurrentPositionAsync({ accuracy: Balanced, timeout: 8000 })
  • Cache {lat,lon,ts,label?} in AsyncStorage for 10 minutes
  • Optional reverse label via backend /geocode with reverse mode
  • Expose { coords, status, refresh, error }; status ∈ "idle"|"checking"|"granted"|"denied"|"error"
- LocationGate screen:
  • If granted→ proceed and fetch with lat/lon
  • If denied→ offer "Enter city manually" (calls /geocode) and "Open settings"
  • If error/timeout→ show retry

UI/UX:
- **Home**: Top‑3 cards
  • Title: time range (e.g., Thu 09:00–12:00)
  • ScoreChip thresholds: green≥0.6, amber≥0.3, red<0.3
  • 2‑line rationale + small “Tip” row with icon
  • 👍/👎 actions (POST /feedback)
- **Timeline**: 7‑day horizontal heatbar (3‑hour bins); tap→ detail drawer with features, rain risk, “Explain”
- **Settings**: "Use device location" toggle; city search; free‑text prefs (POST /prefs & display parsed)
- Icons: @expo/vector-icons (Ionicons/MaterialCommunityIcons) — hanger/shirt, wind, sun, cloud‑rain, thermometer
- Styling: card radius 16–20, spacing 16, subtle shadow, text 13–16, touch targets ≥44 px, dark mode aware
- Colors: slate/neutral base; status: green #22c55e, amber #f59e0b, red #ef4444

Data flow:
- On app start:
  • If device location enabled & granted: use coords and call `/recommendations/top?lat&lon`
  • Else: use last manual city (geocoded) until location allowed
- For visible Top‑3 cards: GET `/explain?window_id=...` lazily (cache by window_id)
- Feedback posts optimistic; revalidate Top‑3

Non‑Functional
--------------
- Type‑safe responses (OpenAPI→TS or Zod mirror types)
- Robust error states: skeletons, retry, clear permission guidance
- Rate‑limit & cache `/explain`
- Tests: scoring, veto, SGD; integration for Top‑3; mock OpenWeather in tests

Acceptance Criteria
-------------------
- `cargo run` starts server on :8080; `/health` ok; `/docs` shows Swagger
- `/geocode` returns lat/lon for city and reverse mode when keys present
- `/recommendations/top?lat&lon` returns ≥3 windows with `unsafe=false` and scores
- `GET /explain?window_id=...` returns STRICT JSON: {rationale, tip, verdict, reasons}
- Expo app `npx expo start` runs; if permission granted, auto‑detects location and shows Top‑3
- Timeline heatbar renders; feedback posts and updates learning
- No secrets committed; `.env.example` exists; README includes run steps

Implement Now
-------------
Generate the full codebase and documentation.

Server constants & behavior:
- Use `OR_MODEL` default "deepseek/deepseek-chat-v3-0324:free"
- OpenRouter call: temperature=0.3, max_tokens=200, response_format json_object
- On invalid AI JSON: return HTTP 502 with `{ error: "invalid_ai_json" }`
- Cache EXPLAIN per `window_id` until forecast refresh

Client:
- `.env.example`: EXPO_PUBLIC_API=http://localhost:8080
- Respect OS dark mode; never embed server secrets
