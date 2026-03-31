//! Dashboard HTTP server implementation
//!
//! Sets up and runs the Axum-based web server for the dashboard.

use axum::{
    routing::get,
    Router,
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use super::routes;
use super::DashboardConfig;
use crate::dashboard::{CampaignSummary, BugReport, DashboardMetrics, CampaignStatus, BugSeverity};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: DashboardConfig,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Start the dashboard web server
pub async fn run_server(config: DashboardConfig) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new(config.clone()));
    
    let app = create_router(state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    println!("Dashboard server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create the Axum router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS layer for cross-origin requests
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Main API routes
    let api_routes = Router::new()
        .route("/health", get(routes::health_check))
        .route("/metrics", get(routes::get_metrics))
        .route("/campaigns", get(routes::list_campaigns))
        .route("/campaigns/:id", get(routes::get_campaign))
        .route("/bugs", get(routes::list_bugs))
        .route("/bugs/:id", get(routes::get_bug));
    
    // Static file serving
    let static_routes = Router::new()
        .nest_service("/", ServeDir::new(&state.config.static_dir).append_index_html_on_directories(true));
    
    Router::new()
        .nest("/api", api_routes)
        .merge(static_routes)
        .layer(cors)
        .with_state(state)
}

/// Root handler - redirect to dashboard
pub async fn root() -> impl IntoResponse {
    Json(json!({
        "name": "ZK Circuit Fuzzer Dashboard",
        "version": "1.0.0",
        "endpoints": {
            "health": "/api/health",
            "metrics": "/api/metrics",
            "campaigns": "/api/campaigns",
            "bugs": "/api/bugs"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let config = DashboardConfig::new();
        let state = AppState::new(config);
        assert_eq!(state.config.port, 8080);
    }
}
