//! Health check endpoint

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    version: String,
}

pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
