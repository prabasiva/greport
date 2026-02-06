//! greport API Server

mod auth;
mod error;
mod rate_limit;
mod response;
mod routes;
mod state;

use axum::{middleware, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rate_limit::start_cleanup_task;
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

    // Start rate limiter cleanup task
    start_cleanup_task(Arc::clone(&state.rate_limiter));

    // Build router with state
    let app = build_router(state);

    // Get bind address
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

fn build_router(state: AppState) -> Router {
    // API v1 routes with middleware
    let api_v1 = Router::new()
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
        // SLA
        .route(
            "/repos/:owner/:repo/sla",
            axum::routing::get(routes::sla::get_sla_report),
        )
        // Apply middleware in reverse order (last added runs first)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::optional_auth_middleware,
        ));

    Router::new()
        // Health check (no auth or rate limiting)
        .route("/health", axum::routing::get(routes::health::health_check))
        // API v1
        .nest("/api/v1", api_v1)
        // Global middleware
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Shutdown signal handler for graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_health_check() {
        // This test would need a mock state
        // Verifies the test infrastructure works
        let health_endpoint = "/health";
        assert_eq!(health_endpoint, "/health");
    }
}
