use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::llm;
use crate::models::{build_response, ScenarioRequest, SimRequest, SimResponse};
use crate::scraper::{self, LiveData};
use crate::sim::{SimConfig, World};
use crate::validation::{validate_elo_overrides, validate_n_sims};

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
    pub glm_api_key: Option<String>,
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
    let results = world.simulate(&config);
    let resp = build_response(&world, &results, &config, None);
    Ok(Json(resp))
}

pub async fn scenario(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScenarioRequest>,
) -> Result<Json<SimResponse>, (StatusCode, String)> {
    let api_key = state.glm_api_key.clone().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "GLM_API_KEY not set — LLM scenario analysis is disabled".to_string(),
    ))?;

    let n_sims =
        validate_n_sims(req.n_sims.unwrap_or(50000)).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

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
    let results = world.simulate(&config);
    let resp = build_response(&world, &results, &config, Some(impact.analysis));
    Ok(Json(resp))
}

pub async fn refresh_live_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LiveData>, (StatusCode, String)> {
    let live = scraper::fetch_all()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Scrape error: {e}")))?;

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
    Ok(Json(live))
}

pub async fn get_live_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<LiveData>>, (StatusCode, String)> {
    let data = state.live_data.read().await.clone();
    Ok(Json(data))
}

pub async fn health() -> &'static str {
    "ok"
}
