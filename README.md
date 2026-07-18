# World Cup 2026 Simulator

[English](README.md) | [Türkçe](README.tr.md)

[![CI](https://github.com/omercagatay/worldcup-2026-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/omercagatay/worldcup-2026-simulator/actions/workflows/ci.yml)

A full-stack Monte Carlo forecast for the 48-team 2026 FIFA World Cup. It combines a Rust simulation engine, a React dashboard, live tournament data, and an optional Kimi-powered scenario analyzer.

> The probabilities and fair odds produced by this project are model estimates, not betting or financial advice.

## Highlights

- Runs 100–200,000 tournament simulations in parallel with Rayon; the dashboard defaults to 50,000.
- Blends Elo, Dixon–Coles, and pi-ratings into expected-goal estimates.
- Uses Dixon–Coles joint score sampling to better represent low-scoring outcomes.
- Locks confirmed group and knockout results, then simulates only the remaining tournament paths.
- Refreshes ratings and tournament results from public live sources on startup and at a configurable interval.
- Calculates title, final, semifinal, quarterfinal, round-of-16, round-of-32, and third-place probabilities.
- Shows group outcomes, a representative bracket, likely final pairings, upcoming-match forecasts, and fair decimal odds.
- Converts natural-language scenarios into validated Elo overrides with Kimi and reruns the tournament.
- Includes per-IP rate limits, request validation, deterministic seeds, light/dark themes, Docker support, and GitHub Actions CI.

## Stack

| Layer | Technology |
|---|---|
| Backend/API | Rust 1.75+, Axum, Tokio |
| Simulation | Rayon, Rand, Dixon–Coles, pi-ratings, Elo/Poisson |
| Frontend | React 18, TypeScript, Vite |
| Live data | World Football Elo Ratings and the English Wikipedia API |
| Scenario analysis | Kimi via the Moonshot API |
| Deployment | Multi-stage Docker image; Railway-compatible |

## How the model works

The pure-Elo component converts rating difference and host advantage into expected goals:

```text
lambda_A = 1.35 × 10^((Elo_A - Elo_B + host_advantage) / 1600)
lambda_B = 1.35 × 10^(-(Elo_A - Elo_B + host_advantage) / 1600)
```

By default, those rates are blended with two models trained from historical international results:

- **Elo (0.5):** current team strength and an 80-point host adjustment.
- **Dixon–Coles (0.3):** time-decayed attack/defence strengths and low-score correlation.
- **Pi-ratings (0.2):** sequential home/away strength updates from match history.

Set `ENSEMBLE_WEIGHTS` to change the blend; `1,0,0` selects the pure-Elo model. When the Dixon–Coles weight is active, regulation-time scorelines use its joint distribution. A tied knockout match proceeds to independently sampled extra time and then, if needed, an Elo-damped penalty shootout.

Live ratings, manual overrides, and Kimi scenarios update the Elo component. The embedded Dixon–Coles and pi-rating parameters stay unchanged until the historical models are explicitly refitted.

The tournament engine applies points, goal difference, goals scored, and head-to-head group tiebreakers. It ranks third-place teams and uses constraint matching against FIFA's eligible round-of-32 slots. Confirmed results are preserved, so eliminated teams cannot re-enter a simulated path.

## Run locally

### Prerequisites

- Rust 1.75 or later
- Node.js 20 or later with npm

The baseline simulator does not require an API key. `KIMI_API_KEY` is needed only for natural-language scenarios.

### Development mode

The Vite development server proxies `/api` to port `3001`, so run the backend on that port.

Terminal 1:

```bash
git clone https://github.com/omercagatay/worldcup-2026-simulator.git
cd worldcup-2026-simulator
cp .env.example .env
PORT=3001 cargo run --release
```

Terminal 2:

```bash
cd worldcup-2026-simulator/frontend
npm ci
npm run dev
```

Open <http://localhost:5173>. The first forecast starts automatically.

### Production-like local build

Build the frontend first; Axum then serves `frontend/dist` together with the API on port `3000`.

```bash
cd frontend
npm ci
npm run build
cd ..
cargo run --release
```

Open <http://localhost:3000>.

## Configuration

Copy `.env.example` to `.env` and adjust these values as needed:

| Variable | Default | Purpose |
|---|---:|---|
| `KIMI_API_KEY` | unset | Enables `/api/scenario`; obtain a key from the Moonshot platform. |
| `PORT` | `3000` | Backend HTTP port. Use `3001` with the Vite development server. |
| `RUST_LOG` | `wc2026_sim=info` | Rust tracing filter. |
| `LIVE_REFRESH_MINUTES` | `30` | Live-data refresh interval; `0` disables background refresh. |
| `ENSEMBLE_WEIGHTS` | `0.5,0.3,0.2` | Comma-separated Elo, Dixon–Coles, and pi-rating weights. |
| `TRUST_PROXY` | `0` | Trust `X-Forwarded-For` for rate limiting only behind a sanitizing reverse proxy. |

## Use the dashboard

1. Choose the simulation count and seed, then select **Run**. Reusing a seed makes the same configuration reproducible.
2. Explore championship forecasts, fair odds, likely finals, the representative bracket, group outcomes, and live tournament data.
3. Select **Update live data** to refresh ratings and confirmed results immediately.
4. Enter a scenario such as `France's starting goalkeeper is unavailable for the final`. Kimi explains the effect, supplies validated team ratings, and starts a new simulation.

## API

| Endpoint | Method | Limit per IP | Description |
|---|---|---:|---|
| `/api/health` | `GET` | — | Service version, model configuration, and last live refresh. |
| `/api/simulate` | `POST` | 30/min | Run a baseline simulation with optional Elo overrides. |
| `/api/scenario` | `POST` | 10/min | Analyze a prompt with Kimi and rerun with its Elo overrides. |
| `/api/refresh` | `POST` | 5/min | Fetch and apply current ratings and tournament results. |
| `/api/live` | `GET` | — | Return the most recently cached live-data snapshot. |
| `/api/upcoming` | `GET` | 30/min | Forecast fixed, unplayed knockout pairings. |

Simulation requests accept 100–200,000 trials. Scenario prompts are limited to 2,000 characters, Elo overrides must name a known team and fall between 1,000 and 2,600, and request bodies are limited to 1 MiB.

### Baseline simulation

```bash
curl -X POST http://localhost:3000/api/simulate \
  -H 'Content-Type: application/json' \
  -d '{"n_sims":50000,"seed":12345}'
```

### Simulation with a manual rating override

`elo_overrides` contains replacement ratings, not point deltas.

```bash
curl -X POST http://localhost:3000/api/simulate \
  -H 'Content-Type: application/json' \
  -d '{"n_sims":50000,"seed":12345,"elo_overrides":{"Turkey":1825}}'
```

### Natural-language scenario

```bash
curl -X POST http://localhost:3000/api/scenario \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"France’s starting goalkeeper is unavailable for the final","n_sims":50000,"seed":12345}'
```

## Docker

```bash
docker build -t wc2026-sim .
docker run --rm -p 3000:3000 \
  -e KIMI_API_KEY=your_key \
  wc2026-sim
```

Omit `KIMI_API_KEY` if scenario analysis is not needed.

## Deploy to Railway

1. Create a Railway service from this GitHub repository.
2. Railway detects the root `Dockerfile` and builds the Rust backend and React frontend.
3. Add `KIMI_API_KEY` if scenario analysis should be enabled.
4. Set `TRUST_PROXY=1` so rate limiting uses the client address supplied by Railway's sanitizing edge proxy.
5. Optionally customize `LIVE_REFRESH_MINUTES`, `ENSEMBLE_WEIGHTS`, and `RUST_LOG`.
6. Set the health-check path to `/api/health`.

The application reads Railway's injected `PORT` automatically.

## Validation

The GitHub Actions workflow runs the same core checks:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release

cd frontend
npm ci
npm run build
```

## Refresh the historical model data

The repository already includes the historical results and fitted Dixon–Coles parameters used at build time. To refresh and refit them:

```bash
./scripts/refresh_history.sh
cargo run --release --example fit_dc
```

Review and validate the changed files under `data/` before committing a new fit.

## Project structure

```text
.
├── src/
│   ├── main.rs           # Axum server, configuration, and background refresh
│   ├── sim.rs            # Tournament and parallel Monte Carlo engine
│   ├── dixoncoles.rs     # Dixon–Coles fitting and joint score probabilities
│   ├── piratings.rs      # Historical pi-rating model
│   ├── history.rs        # Historical result loading and team normalization
│   ├── scraper.rs        # Live rating and tournament-data ingestion
│   ├── handlers.rs       # API handlers
│   ├── llm.rs            # Kimi scenario analysis
│   ├── models.rs         # API request and response types
│   ├── validation.rs     # Request validation
│   └── rate_limit.rs     # Per-IP rate limiting
├── data/                 # Historical results and fitted model parameters
├── frontend/             # React and TypeScript dashboard
├── examples/             # Model fitting and smoke-test utilities
├── scripts/              # Data-refresh helpers
├── .github/workflows/    # CI configuration
└── Dockerfile            # Production multi-stage image
```

## Data and model caveats

- Live refresh depends on third-party endpoints and their current page/data formats; the embedded baseline remains available if a refresh fails.
- Fair odds are simply the inverse of simulated probabilities and do not include a bookmaker margin, liquidity, or market information.
- Scenario ratings are model-generated assumptions. Read the returned explanation and treat the output as exploratory.
- Forecast quality depends on ratings, historical-data coverage, model assumptions, and the number of trials.
