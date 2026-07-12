use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir};
use tracing_subscriber::EnvFilter;

use tokio::sync::RwLock;
use wc2026_sim::{
    handlers::{self, AppState},
    rate_limit::RateLimitLayer,
    sim::World,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("wc2026_sim=info".parse()?))
        .init();

    let world = World::new();
    let kimi_api_key = std::env::var("KIMI_API_KEY").ok();
    if kimi_api_key.is_some() {
        tracing::info!("Kimi scenario analysis enabled");
    } else {
        tracing::warn!("KIMI_API_KEY not set — scenario endpoint will return an error");
    }

    let state = Arc::new(AppState {
        world: Arc::new(RwLock::new(world)),
        kimi_api_key,
        live_data: Arc::new(RwLock::new(None)),
    });

    // Keep the simulation current with the real tournament: refresh live
    // data immediately on startup, then on an interval. LIVE_REFRESH_MINUTES=0
    // disables the background task (manual /api/refresh still works).
    let refresh_minutes: u64 = std::env::var("LIVE_REFRESH_MINUTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    if refresh_minutes > 0 {
        let bg_state = state.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(refresh_minutes * 60));
            loop {
                interval.tick().await;
                match handlers::perform_live_refresh(&bg_state).await {
                    Ok(live) => tracing::info!(
                        "Background live refresh ok: {} group matches, {} knockout matches",
                        live.played_matches.len(),
                        live.knockout_matches.len()
                    ),
                    Err(e) => tracing::warn!("Background live refresh failed: {e:#}"),
                }
            }
        });
        tracing::info!("Background live refresh enabled (every {refresh_minutes} min)");
    } else {
        tracing::info!("Background live refresh disabled (LIVE_REFRESH_MINUTES=0)");
    }

    let app = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/live", get(handlers::get_live_data))
        .merge(
            Router::new()
                .route("/api/simulate", post(handlers::run_sim))
                .route_layer(RateLimitLayer::new(30, 60)),
        )
        .merge(
            Router::new()
                .route("/api/scenario", post(handlers::scenario))
                .route_layer(RateLimitLayer::new(10, 60)),
        )
        .merge(
            Router::new()
                .route("/api/refresh", post(handlers::refresh_live_data))
                .route_layer(RateLimitLayer::new(5, 60)),
        )
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(ServeDir::new("frontend/dist"));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
