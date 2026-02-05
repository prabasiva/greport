//! greport API Server

mod error;
mod response;
mod routes;
mod state;

use axum::Router;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create application state
    let state = AppState::new().await?;

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", axum::routing::get(routes::health::health_check))
        // API v1
        .nest("/api/v1", api_v1_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Get bind address
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Issues
        .route(
            "/repos/:owner/:repo/issues",
            axum::routing::get(routes::issues::list_issues),
        )
        .route(
            "/repos/:owner/:repo/issues/metrics",
            axum::routing::get(routes::issues::get_metrics),
        )
        .route(
            "/repos/:owner/:repo/issues/velocity",
            axum::routing::get(routes::issues::get_velocity),
        )
        .route(
            "/repos/:owner/:repo/issues/burndown",
            axum::routing::get(routes::issues::get_burndown),
        )
        .route(
            "/repos/:owner/:repo/issues/stale",
            axum::routing::get(routes::issues::get_stale),
        )
        // Pull Requests
        .route(
            "/repos/:owner/:repo/pulls",
            axum::routing::get(routes::pulls::list_pulls),
        )
        .route(
            "/repos/:owner/:repo/pulls/metrics",
            axum::routing::get(routes::pulls::get_metrics),
        )
        // Releases
        .route(
            "/repos/:owner/:repo/releases",
            axum::routing::get(routes::releases::list_releases),
        )
        .route(
            "/repos/:owner/:repo/releases/notes",
            axum::routing::get(routes::releases::get_notes),
        )
        .route(
            "/repos/:owner/:repo/milestones/:milestone/progress",
            axum::routing::get(routes::releases::get_progress),
        )
        // Contributors
        .route(
            "/repos/:owner/:repo/contributors",
            axum::routing::get(routes::contrib::list_contributors),
        )
}
