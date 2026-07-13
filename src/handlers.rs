use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::llm;
use crate::models::{build_response, ScenarioRequest, SimRequest, SimResponse};
use crate::scraper::{self, LiveData};
use crate::sim::{SimConfig, World};
use crate::validation::{validate_elo_overrides, validate_n_sims, validate_prompt};

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
    pub kimi_api_key: Option<String>,
    pub live_data: Arc<RwLock<Option<LiveData>>>,
}

pub async fn run_sim(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SimRequest>,
) -> Result<Json<SimResponse>, (StatusCode, String)> {
    let n_sims =
        validate_n_sims(req.n_sims.unwrap_or(50000)).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let config = SimConfig {
        n_sims,
        seed: req.seed.unwrap_or(12345),
        elo_overrides: req.elo_overrides.unwrap_or_default(),
    };
    let world = {
        let w = state.world.read().await.clone();
        if config.elo_overrides.is_empty() {
            w
        } else {
            let mut w = w;
            w.apply_overrides(&config.elo_overrides);
            w
        }
    };
    let resp = simulate_off_runtime(world, config, None).await?;
    Ok(Json(resp))
}

/// Run the CPU-bound rayon simulation on the blocking pool so it can't
/// stall tokio's worker threads (and with them /api/health).
async fn simulate_off_runtime(
    world: crate::sim::World,
    config: SimConfig,
    scenario: Option<String>,
) -> Result<SimResponse, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let results = world.simulate(&config);
        build_response(&world, &results, &config, scenario)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Simulation task failed: {e}"),
        )
    })
}

pub async fn scenario(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScenarioRequest>,
) -> Result<Json<SimResponse>, (StatusCode, String)> {
    let api_key = state.kimi_api_key.clone().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "KIMI_API_KEY not set — LLM scenario analysis is disabled".to_string(),
    ))?;

    let n_sims =
        validate_n_sims(req.n_sims.unwrap_or(50000)).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_prompt(&req.prompt).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let impact = llm::analyze_scenario(&req.prompt, &api_key)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("LLM error: {e}")))?;

    let world = state.world.read().await.clone();
    validate_elo_overrides(&world, &impact.adjustments)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let mut world = world;
    world.apply_overrides(&impact.adjustments);

    let config = SimConfig {
        n_sims,
        seed: req.seed.unwrap_or(12345),
        elo_overrides: impact.adjustments.clone(),
    };
    let resp = simulate_off_runtime(world, config, Some(impact.analysis)).await?;
    Ok(Json(resp))
}

/// Scrape live data, apply it to the shared `World`, and cache the raw
/// result. Shared by the `/api/refresh` handler and the background
/// refresh task spawned in `main.rs`.
pub async fn perform_live_refresh(state: &AppState) -> anyhow::Result<LiveData> {
    let live = scraper::fetch_all().await?;

    let (elo_n, match_n) = {
        let mut world = state.world.write().await;
        world.update_from_live(&live)
    };

    tracing::info!(
        "Live data applied to simulation: {} Elo ratings, {} matches",
        elo_n,
        match_n
    );

    *state.live_data.write().await = Some(live.clone());
    Ok(live)
}

pub async fn refresh_live_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LiveData>, (StatusCode, String)> {
    let live = perform_live_refresh(&state)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Scrape error: {e}")))?;
    Ok(Json(live))
}

pub async fn get_live_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<LiveData>>, (StatusCode, String)> {
    let data = state.live_data.read().await.clone();
    Ok(Json(data))
}

/// Forecasts for real bracket matches whose pairing is fixed but which
/// haven't been played yet (currently the semifinals; later the third-place
/// match and final).
pub async fn upcoming(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::models::UpcomingResponse>, (StatusCode, String)> {
    let world = state.world.read().await.clone();
    let resp = tokio::task::spawn_blocking(move || {
        let matches = world
            .upcoming_matches()
            .into_iter()
            .map(|(match_id, round, ta, tb)| {
                let (a_win_pct, b_win_pct, decided_in_90_pct) =
                    world.match_win_probs(ta, tb, 100_000, 12345);
                crate::models::UpcomingMatch {
                    match_id,
                    round: round.to_string(),
                    team_a: world.teams[ta].clone(),
                    team_b: world.teams[tb].clone(),
                    a_win_pct,
                    b_win_pct,
                    decided_in_90_pct,
                    a_win_odds: crate::odds::decimal_odds_from_pct(a_win_pct),
                    b_win_odds: crate::odds::decimal_odds_from_pct(b_win_pct),
                }
            })
            .collect();
        crate::models::UpcomingResponse { matches }
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Upcoming forecast task failed: {e}"),
        )
    })?;
    Ok(Json(resp))
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let live_fetched_at = state
        .live_data
        .read()
        .await
        .as_ref()
        .map(|l| l.fetched_at.clone());
    let model = {
        let world = state.world.read().await;
        match &world.ensemble {
            Some(e) => serde_json::json!({
                "kind": "ensemble",
                "weights": { "elo": e.w_elo, "dixon_coles": e.w_dc, "pi_ratings": e.w_pi },
                "dc_fitted_at": e.dc.fitted_at,
                "pi_matches": e.pi.n_matches,
                "score_sampler": if e.w_dc > 0.0 { "dixon_coles_joint" } else { "independent_poisson" },
            }),
            None => serde_json::json!({ "kind": "elo", "score_sampler": "independent_poisson" }),
        }
    };
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "live_fetched_at": live_fetched_at,
        "model": model,
    }))
}
