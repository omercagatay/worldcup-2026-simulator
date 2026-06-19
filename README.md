# WC 2026 Monte Carlo Simulator

Monte Carlo simulation of the 2026 FIFA World Cup (48-team format) with an LLM-powered scenario engine.

- **Backend**: Rust (axum) — Elo → Poisson match model, parallel Monte Carlo simulation via rayon
- **Frontend**: TypeScript (Vite + React) — results tables, group standings, bracket viz
- **LLM**: GLM-5.2 (Z.ai API) — natural-language scenarios → structured Elo adjustments → re-simulation

## Model

Each match outcome is modeled as independent Poisson goals, with expected goals (λ) derived from the Elo rating difference:

```
λ_a = 1.35 × 10^((Elo_a - Elo_b + home_adv) / 1600)
λ_b = 1.35 × 10^(-(Elo_a - Elo_b + home_adv) / 1600)
```

Knockout matches that are tied after 90 min get extra time (λ × 0.5), then penalties with a damped Elo-based probability.

Group stage: round-robin, points/GD/GF ranking with head-to-head tiebreakers. Third-place qualification uses backtracking constraint matching against FIFA's slot eligibility table. Already-played first-round matches are fixed results.

## Setup

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 18+

### Backend

```bash
cd wc2026-sim
cp .env.example .env          # add your GLM_API_KEY (get one at https://z.ai)
cargo run --release            # serves on http://localhost:3000
```

### Frontend

```bash
cd wc2026-sim/frontend
npm install
npm run dev                    # serves on http://localhost:5173 (proxies /api to backend)
```

Open http://localhost:5173 in your browser.

## Usage

1. **Run baseline simulation**: Click "Run Simulation" (default: 20,000 sims)
2. **Apply a scenario**: Type a natural-language prompt like:
   - "Lamine Yamal gets injured in Spain's second group match"
   - "Mbappe is suspended for the knockout stage"
   - "Argentina's entire defense has food poisoning"

   The LLM analyzes the prompt, adjusts the affected team's Elo rating, and re-runs the full tournament simulation with the new ratings.

## API

| Endpoint | Method | Description |
|---|---|---|
| `/api/health` | GET | Health check |
| `/api/simulate` | POST | Run simulation with optional Elo overrides (100–200,000 sims) |
| `/api/scenario` | POST | LLM-analyze prompt, adjust Elo, re-run simulation (rate limited) |
| `/api/refresh` | POST | Scrape live Elo ratings and results from Wikipedia |
| `/api/live` | GET | Return cached live data |

### Example

```bash
curl -X POST http://localhost:3000/api/simulate \
  -H 'Content-Type: application/json' \
  -d '{"n_sims": 20000, "seed": 12345}'
```

## Deploy

### Railway

1. Push the repo to GitHub
2. In [Railway](https://railway.app), click **New Service → Deploy from GitHub repo**
3. Railway auto-detects the `Dockerfile`
4. Add environment variables in the Railway dashboard:
   - `GLM_API_KEY` — your Z.ai API key (required for /api/scenario)
   - `PORT` — defaults to 3000
   - `RUST_LOG` — `wc2026_sim=info`
5. Set health check path to `/api/health`

Railway will build the Docker image (Rust + frontend) and deploy on a public URL with automatic SSL.

### Docker (manual)

```bash
docker build -t wc2026-sim .
docker run -p 3000:3000 -e GLM_API_KEY=your_key wc2026-sim
```

## Project Structure

```
wc2026-sim/
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs          # axum server entry point
│   ├── data.rs          # WC 2026 data (teams, Elo, groups, bracket)
│   ├── sim.rs           # Monte Carlo simulation core
│   ├── models.rs        # request/response types, response builder
│   ├── handlers.rs      # HTTP handlers
│   ├── llm.rs           # GLM-5.2 integration (Z.ai API)
│   ├── scraper.rs       # Live Elo/results/standings scraping
│   ├── validation.rs    # Request validation helpers
│   └── rate_limit.rs    # Per-IP rate limiting middleware
└── frontend/
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── App.tsx
        ├── api.ts
        ├── styles.css
        └── components/
            ├── ResultsTable.tsx
            ├── GroupTables.tsx
            ├── BracketView.tsx
            ├── ScenarioPrompt.tsx
            └── LiveStats.tsx
```
