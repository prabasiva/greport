//! Application state

use crate::rate_limit::RateLimiter;
use greport_core::OctocrabClient;
use greport_db::DbPool;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// GitHub client
    pub github: Arc<OctocrabClient>,
    /// API configuration
    pub config: Arc<ApiConfig>,
    /// Database pool (optional)
    pub db: Option<DbPool>,
    /// Rate limiter
    pub rate_limiter: Arc<RateLimiter>,
}

/// API configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApiConfig {
    /// Rate limit per minute per client (default)
    pub rate_limit_per_minute: u32,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum page size
    pub max_page_size: usize,
    /// Enable authentication requirement
    pub require_auth: bool,
    /// SLA response time threshold in hours
    pub sla_response_hours: i64,
    /// SLA resolution time threshold in hours
    pub sla_resolution_hours: i64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 60,
            cache_ttl_seconds: 300,
            max_page_size: 100,
            require_auth: false,
            sla_response_hours: 24,
            sla_resolution_hours: 168, // 1 week
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
            require_auth: std::env::var("REQUIRE_AUTH")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            sla_response_hours: std::env::var("SLA_RESPONSE_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            sla_resolution_hours: std::env::var("SLA_RESOLUTION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(168),
        }
    }
}

impl AppState {
    /// Create new application state
    pub async fn new() -> anyhow::Result<Self> {
        // Get GitHub token and optional base URL from environment
        tracing::debug!("Initializing application state");

        let token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| anyhow::anyhow!("GITHUB_TOKEN environment variable not set"))?;
        tracing::debug!("GITHUB_TOKEN found in environment");

        let base_url = std::env::var("GITHUB_BASE_URL").ok();
        if let Some(ref url) = base_url {
            tracing::info!(base_url = %url, "Using GitHub Enterprise base URL");
        } else {
            tracing::debug!("Using default GitHub.com API");
        }

        let github = Arc::new(OctocrabClient::new(&token, base_url.as_deref())?);
        tracing::info!("GitHub client initialized");

        let config = Arc::new(ApiConfig::from_env());
        tracing::debug!(
            rate_limit = config.rate_limit_per_minute,
            cache_ttl = config.cache_ttl_seconds,
            "API configuration loaded"
        );

        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_minute));

        // Try to connect to database (optional)
        let db = match greport_db::create_pool().await {
            Ok(pool) => {
                tracing::info!("Connected to database");
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("Database not available: {}. Running without caching.", e);
                None
            }
        };

        Ok(Self {
            github,
            config,
            db,
            rate_limiter,
        })
    }

    /// Create state with custom configuration
    #[allow(dead_code)]
    pub async fn with_config(config: ApiConfig) -> anyhow::Result<Self> {
        let github = Arc::new(OctocrabClient::from_env()?);
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_minute));
        let db = greport_db::create_pool().await.ok();

        Ok(Self {
            github,
            config: Arc::new(config),
            db,
            rate_limiter,
        })
    }

    /// Create state for testing (no database)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn test_state(github: OctocrabClient) -> Self {
        let config = ApiConfig::default();
        Self {
            github: Arc::new(github),
            config: Arc::new(config.clone()),
            db: None,
            rate_limiter: Arc::new(RateLimiter::new(config.rate_limit_per_minute)),
        }
    }
}
