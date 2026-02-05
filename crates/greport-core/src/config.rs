//! Configuration management for greport

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            defaults: DefaultsConfig::default(),
            sla: SlaConfig::default(),
        }
    }
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
}
