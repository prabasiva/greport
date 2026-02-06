//! Configuration management for greport

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// GitHub configuration
    #[serde(default)]
    pub github: GitHubConfig,

    /// Default settings
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// SLA configuration
    #[serde(default)]
    pub sla: SlaConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// API server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// GitHub-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub personal access token (optional, can use env var)
    pub token: Option<String>,

    /// GitHub API base URL (for GitHub Enterprise)
    pub base_url: Option<String>,
}

/// Default settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default repository (owner/repo)
    pub repo: Option<String>,

    /// Default output format
    #[serde(default = "default_format")]
    pub format: String,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

/// SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    /// Default response time in hours
    #[serde(default = "default_response_time")]
    pub response_time_hours: i64,

    /// Default resolution time in hours
    #[serde(default = "default_resolution_time")]
    pub resolution_time_hours: i64,

    /// Priority-specific overrides
    #[serde(default)]
    pub priority: HashMap<String, SlaPriority>,
}

/// Priority-specific SLA settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaPriority {
    /// Response time in hours
    pub response_time_hours: i64,

    /// Resolution time in hours
    pub resolution_time_hours: i64,
}

/// Database configuration (used by API server)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL (postgres://...)
    /// Can also be set via DATABASE_URL env var (takes priority)
    pub url: Option<String>,

    /// Maximum connections in pool
    pub max_connections: Option<u32>,

    /// Connection acquire timeout in seconds
    pub acquire_timeout_secs: Option<u64>,

    /// Run migrations on connect
    pub run_migrations: Option<bool>,
}

/// API server configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind host address
    /// Can also be set via API_HOST env var (takes priority)
    pub host: Option<String>,

    /// Bind port
    /// Can also be set via API_PORT env var (takes priority)
    pub port: Option<u16>,

    /// Rate limit per minute per client
    pub rate_limit_per_minute: Option<u32>,

    /// Cache TTL in seconds for API responses
    pub cache_ttl_seconds: Option<u64>,

    /// Maximum page size for paginated responses
    pub max_page_size: Option<usize>,

    /// Require API key authentication
    pub require_auth: Option<bool>,
}

/// Logging configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter string (e.g., "info", "debug", "info,tower_http=debug")
    /// Can also be set via RUST_LOG env var (takes priority)
    pub level: Option<String>,
}

impl Default for SlaConfig {
    fn default() -> Self {
        let mut priority = HashMap::new();
        priority.insert(
            "critical".to_string(),
            SlaPriority {
                response_time_hours: 4,
                resolution_time_hours: 24,
            },
        );
        priority.insert(
            "high".to_string(),
            SlaPriority {
                response_time_hours: 8,
                resolution_time_hours: 72,
            },
        );

        Self {
            response_time_hours: default_response_time(),
            resolution_time_hours: default_resolution_time(),
            priority,
        }
    }
}

fn default_format() -> String {
    "table".to_string()
}

fn default_cache_ttl() -> u64 {
    3600
}

fn default_response_time() -> i64 {
    24
}

fn default_resolution_time() -> i64 {
    168 // 7 days
}

impl Config {
    /// Load configuration from file
    pub fn load(path: Option<&PathBuf>) -> crate::Result<Self> {
        let config_path = match path {
            Some(p) => p.clone(),
            None => Self::default_config_path()?,
        };

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| crate::Error::Config(format!("Failed to read config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> crate::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| crate::Error::Config("Could not determine home directory".into()))?;

        Ok(home.join(".config").join("greport").join("config.toml"))
    }

    /// Get GitHub token from config or environment
    pub fn github_token(&self) -> crate::Result<String> {
        if let Some(token) = &self.github.token {
            return Ok(token.clone());
        }

        std::env::var("GITHUB_TOKEN").map_err(|_| crate::Error::MissingToken)
    }

    /// Resolve database URL (env var > config file)
    pub fn database_url(&self) -> Option<String> {
        std::env::var("DATABASE_URL")
            .ok()
            .or_else(|| self.database.url.clone())
    }

    /// Resolve API host (env var > config file > "0.0.0.0")
    pub fn api_host(&self) -> String {
        std::env::var("API_HOST")
            .ok()
            .or_else(|| self.server.host.clone())
            .unwrap_or_else(|| "0.0.0.0".to_string())
    }

    /// Resolve API port (env var > config file > 9423)
    pub fn api_port(&self) -> u16 {
        std::env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.server.port)
            .unwrap_or(9423)
    }

    /// Resolve RUST_LOG (env var > config file > provided default)
    pub fn rust_log(&self, default: &str) -> String {
        std::env::var("RUST_LOG")
            .ok()
            .or_else(|| self.logging.level.clone())
            .unwrap_or_else(|| default.to_string())
    }

    /// Resolve database max connections (env var > config file > 10)
    pub fn db_max_connections(&self) -> u32 {
        std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.database.max_connections)
            .unwrap_or(10)
    }

    /// Resolve database acquire timeout (env var > config file > 5)
    pub fn db_acquire_timeout_secs(&self) -> u64 {
        std::env::var("DB_ACQUIRE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.database.acquire_timeout_secs)
            .unwrap_or(5)
    }

    /// Resolve whether to run migrations (env var > config file > true)
    pub fn db_run_migrations(&self) -> bool {
        if let Ok(v) = std::env::var("DB_RUN_MIGRATIONS") {
            return v != "false" && v != "0";
        }
        self.database.run_migrations.unwrap_or(true)
    }

    /// Resolve rate limit per minute (env var > config file > 60)
    pub fn rate_limit_per_minute(&self) -> u32 {
        std::env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.server.rate_limit_per_minute)
            .unwrap_or(60)
    }

    /// Resolve cache TTL seconds (env var > config file > 300)
    pub fn cache_ttl_seconds(&self) -> u64 {
        std::env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.server.cache_ttl_seconds)
            .unwrap_or(300)
    }

    /// Resolve max page size (env var > config file > 100)
    pub fn max_page_size(&self) -> usize {
        std::env::var("MAX_PAGE_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .or(self.server.max_page_size)
            .unwrap_or(100)
    }

    /// Resolve require auth (env var > config file > false)
    pub fn require_auth(&self) -> bool {
        if let Ok(v) = std::env::var("REQUIRE_AUTH") {
            return v == "true" || v == "1";
        }
        self.server.require_auth.unwrap_or(false)
    }
}
