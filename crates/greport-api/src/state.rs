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
    /// Load config with priority: env var > config.toml > defaults
    pub fn from_core_config(config: &greport_core::Config) -> Self {
        Self {
            rate_limit_per_minute: config.rate_limit_per_minute(),
            cache_ttl_seconds: config.cache_ttl_seconds(),
            max_page_size: config.max_page_size(),
            require_auth: config.require_auth(),
            sla_response_hours: std::env::var("SLA_RESPONSE_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(config.sla.response_time_hours),
            sla_resolution_hours: std::env::var("SLA_RESOLUTION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(config.sla.resolution_time_hours),
        }
    }

}

impl AppState {
    /// Create application state from a pre-loaded Config
    pub async fn with_core_config(core_config: greport_core::Config) -> anyhow::Result<Self> {
        tracing::debug!("Initializing application state");

        let token = core_config
            .github_token()
            .map_err(|_| anyhow::anyhow!("GITHUB_TOKEN not set in environment or ~/.config/greport/config.toml"))?;
        tracing::debug!("GitHub token loaded");

        let base_url = std::env::var("GITHUB_BASE_URL")
            .ok()
            .or(core_config.github.base_url.clone());
        if let Some(ref url) = base_url {
            tracing::info!(base_url = %url, "Using GitHub Enterprise base URL");
        } else {
            tracing::debug!("Using default GitHub.com API");
        }

        let github = Arc::new(OctocrabClient::new(&token, base_url.as_deref())?);
        tracing::info!("GitHub client initialized");

        let config = Arc::new(ApiConfig::from_core_config(&core_config));
        tracing::debug!(
            rate_limit = config.rate_limit_per_minute,
            cache_ttl = config.cache_ttl_seconds,
            "API configuration loaded"
        );

        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_minute));

        // Try to connect to database (optional)
        let db = match core_config.database_url() {
            Some(url) => {
                let db_config = greport_db::DbConfig {
                    database_url: url,
                    max_connections: core_config.db_max_connections(),
                    acquire_timeout_secs: core_config.db_acquire_timeout_secs(),
                    run_migrations: core_config.db_run_migrations(),
                };
                match greport_db::create_pool_with_config(&db_config).await {
                    Ok(pool) => {
                        tracing::info!("Connected to database");
                        Some(pool)
                    }
                    Err(e) => {
                        tracing::warn!("Database connection failed: {}. Running without caching.", e);
                        None
                    }
                }
            }
            None => {
                tracing::info!("No DATABASE_URL configured. Running without database.");
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
