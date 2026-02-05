//! Application state

use greport_core::OctocrabClient;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// GitHub client
    pub github: Arc<OctocrabClient>,
    /// API configuration
    pub config: Arc<ApiConfig>,
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Rate limit per minute per client
    pub rate_limit_per_minute: u32,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum page size
    pub max_page_size: usize,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 60,
            cache_ttl_seconds: 300,
            max_page_size: 100,
        }
    }
}

impl ApiConfig {
    /// Load config from environment
    pub fn from_env() -> Self {
        Self {
            rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            cache_ttl_seconds: std::env::var("CACHE_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            max_page_size: std::env::var("MAX_PAGE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        }
    }
}

impl AppState {
    /// Create new application state
    pub async fn new() -> anyhow::Result<Self> {
        let github = Arc::new(OctocrabClient::from_env()?);
        let config = Arc::new(ApiConfig::from_env());

        Ok(Self { github, config })
    }
}
