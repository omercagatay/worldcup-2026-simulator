# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Monte Carlo simulator for the 2026 FIFA World Cup (48-team format). Rust/axum backend runs the simulation and serves a built React frontend as static files; an LLM (Kimi/Moonshot) turns natural-language "what if" scenarios into Elo adjustments that get re-simulated.

## Commands

### Backend (Rust, repo root)

```bash
cargo run --release             # serve on :3000 (reads .env for KIMI_API_KEY, PORT, RUST_LOG)
cargo test                      # all tests (mostly inline #[cfg(test)] in src/sim.rs, src/validation.rs)
cargo test <test_name>          # single test, e.g. cargo test simulate_is_deterministic_for_same_seed
cargo fmt -- --check            # CI formatting check
cargo clippy --all-targets -- -D warnings   # CI lint (warnings are hard errors)
cargo build --release
cargo run --example fit_dc      # offline Dixon-Coles fit against data/international_results.csv
```

### Frontend (`frontend/`)

```bash
npm install
npm run dev            # Vite dev server on :5173, proxies /api to http://localhost:3001 (note: not :3000 — start the backend with PORT=3001 for the dev proxy to work, or run `npm run build` and hit the backend directly on :3000)
npx tsc --noEmit        # CI type check
npm run build           # tsc + vite build -> frontend/dist (served by the Rust binary in prod/docker)
```

CI (`.github/workflows/ci.yml`) runs both jobs independently: backend (`fmt`, `clippy -D warnings`, `test`, `build --release`) and frontend (`tsc --noEmit`, `build`).

### Docker

`Dockerfile` is a 3-stage build: Rust backend builder (with a dependency-caching dummy-`main.rs` layer) → Node frontend builder → slim Debian runtime that copies the backend binary and `frontend/dist`. Deploys to Railway by auto-detecting the Dockerfile; health check path is `/api/health`.

## Architecture

### Request flow

`src/main.rs` builds one `AppState` (`Arc<AppState>`) holding:
- `world: Arc<RwLock<World>>` — the live simulation state (teams, Elo ratings, groups, already-played matches)
- `live_data: Arc<RwLock<Option<LiveData>>>` — cached scrape results
- `kimi_api_key: Option<String>`

Routes (`src/handlers.rs`), each with its own per-IP rate limit (`src/rate_limit.rs`, sliding window):
- `POST /api/simulate` (30/min) — takes optional `elo_overrides`, clones the current `World`, applies overrides **to the clone only** (does not mutate shared state), runs `World::simulate`.
- `POST /api/scenario` (10/min) — sends the prompt to Kimi (`src/llm.rs`), validates the returned Elo adjustments against team names/bounds (`src/validation.rs`), applies them to a cloned `World`, simulates, returns results plus the LLM's `analysis` text.
- `POST /api/refresh` (5/min) — scrapes Wikipedia (`src/scraper.rs`) for live Elo ratings, group results, and knockout results, then **does** mutate the shared `World` (`world.update_from_live`) and caches the raw scrape in `live_data`. This is the only path that changes state for subsequent requests.
- `GET /api/upcoming` (30/min) — win probabilities for real bracket matches whose pairing is fixed but not yet played (semifinals → third-place match → final), computed per-match via `World::match_win_probs`.
- `GET /api/live`, `GET /api/health` — read-only.

Everything not matching `/api/*` falls back to `ServeDir::new("frontend/dist")`, so in production this is a single binary serving both API and SPA.

### Simulation core (`src/sim.rs`)

`World::simulate` runs `n_sims` independent trials in parallel via `rayon`, each seeded deterministically (`config.seed.wrapping_add(i * 2654435761)`) so results are reproducible for a given seed — this determinism is asserted in tests, don't break it.

Per trial (`simulate_one`):
1. Each group's 6 matches are simulated (or taken from `World.played` if already recorded) as independent Poisson goal counts, λ derived from the Elo difference plus a home-advantage term (`src/data.rs`: `BASE`, `D_DIV`, `HOME_ADV`).
2. Groups are ranked by points → GD → GF, with head-to-head sub-sorting for tied blocks (`rank_group`).
3. The 8 best third-place teams qualify; which third-placed group fills which knockout slot is solved via backtracking against FIFA's slot-eligibility table (`assign_thirds` / `data::third_place_slots`).
4. Knockout matches (`ko_match`) add extra time (λ × `ET_FACTOR`) and, if still tied, a damped Elo-weighted penalty-shootout probability. Any match already recorded in `World.played_knockout` short-circuits simulation and returns the real result.
5. Bracket progression is driven purely by the match-ID graph in `src/data.rs` (`r32`/`r16`/`qf`/`sf`/`FINAL`), not by hardcoded team logic.

Across all trials, `simulate` aggregates counts (champion, finalist, SF/QF/R16/R32 appearances, group finishing position) and also picks a single "representative" bracket — the trial whose slot winners most often match the per-slot mode outcome — used for the bracket visualization in the UI.

### Data layer (`src/data.rs`)

Static tournament structure: team list + Elo, group assignments, host nations (home advantage), already-played first-round results (fixed, not simulated), and the knockout bracket graph. Changing team Elo, groups, or results for a new tournament stage means editing this file (or applying `elo_overrides` / scraping live data at request time).

### Strength-model ensemble (`src/dixoncoles.rs`, `src/piratings.rs`, `src/history.rs`)

The simulation blends three strength models into each match's expected goals (λ), weighted by `ENSEMBLE_WEIGHTS` env var (`"elo,dc,pi"`, default `0.5,0.3,0.2`; `1,0,0` = pure Elo):
- **Elo-Poisson** (original model): λ from Elo difference via `BASE`/`D_DIV`/`HOME_ADV`.
- **Dixon-Coles** attack/defense params, fit offline against `data/international_results.csv` (refreshed via `scripts/refresh_history.sh`) with `cargo run --release --example fit_dc`, which writes `data/dc_params.json`; the server loads that file via `include_str!` at startup (~2s refit when the team list or history changes). The runtime uses DC λs only, not the ρ low-score correction.
- **Pi-ratings** (Constantinou–Fenton), computed in one fast pass over the same history at startup (`src/piratings.rs`).

`World.ensemble: Option<Ensemble>` holds the blend; `None` (as in `World::new()` and most tests) means pure Elo. Team indices in DC/pi coincide with `World` indices because `history::TeamIndex::wc()` is built from the same `data::elo()` order (plus a trailing "Rest of World" bucket). Elo overrides from scenarios act through the Elo component only. `GET /api/health` reports the active model and weights. The penalty-shootout model stays Elo-based.

### LLM scenario analysis (`src/llm.rs`)

Calls the Kimi/Moonshot chat completions API (`kimi-k2.6`, thinking disabled for latency) with a system prompt that enumerates all 48 valid team names and Elo-adjustment heuristics (injury/suspension point ranges). Expects strict JSON back (`{"analysis": ..., "adjustments": {...}}`); `strip_fences` tolerates markdown code fences some models add. If a team name isn't in the canonical list the LLM is instructed to omit it — `validate_elo_overrides` still re-checks this server-side since the LLM output isn't trusted.

### Frontend (`frontend/src/`)

`api.ts` mirrors the backend's JSON response shapes exactly (`SimResponse`, `TeamRow`, `GroupRow`, `BracketSlot`, `LiveData`, etc.) — when changing `src/models.rs` response structs, update `api.ts` in the same change. `App.tsx` owns simulation state and drives the child components (`ResultsTable`, `GroupTables`, `BracketView`, `ScenarioPrompt`, `LiveStats`); there's no separate state management library.
