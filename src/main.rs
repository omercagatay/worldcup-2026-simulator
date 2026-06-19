mod data;
mod handlers;
mod llm;
mod models;
mod rate_limit;
mod scraper;
mod sim;
mod validation;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    services::ServeDir,
};
use tracing_subscriber::EnvFilter;

use rate_limit::RateLimitLayer;

use handlers::AppState;
use sim::World;
use tokio::sync::RwLock;

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
    axum::serve(listener, app).await?;
    Ok(())
}
