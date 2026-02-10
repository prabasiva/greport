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

    /// Organization-specific configurations
    #[serde(default)]
    pub organizations: Vec<OrgConfig>,

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

/// Single organization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgConfig {
    /// GitHub organization name (e.g., "my-org")
    pub name: String,
    /// GitHub personal access token scoped to this organization
    pub token: String,
    /// Optional GitHub API base URL (for GitHub Enterprise)
    pub base_url: Option<String>,
    /// Optional list of repository names (without org prefix) to report on
    #[serde(default)]
    pub repos: Option<Vec<String>>,
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

/// Mask a token for display: shows first 4 chars + `****` + last 4 chars.
/// For tokens with 8 or fewer characters, shows first 4 + `****`.
pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        let prefix: String = token.chars().take(4).collect();
        format!("{}****", prefix)
    } else {
        let prefix: String = token.chars().take(4).collect();
        let suffix: String = token
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{}****{}", prefix, suffix)
    }
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

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))?;

        config.merge_org_env_vars();

        Ok(config)
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

    /// Scan environment for GREPORT_ORG_*_TOKEN variables and merge into config
    pub fn merge_org_env_vars(&mut self) {
        for (key, value) in std::env::vars() {
            if let Some(org_suffix) = key.strip_prefix("GREPORT_ORG_") {
                if let Some(org_upper) = org_suffix.strip_suffix("_TOKEN") {
                    let org_name = org_upper.to_lowercase().replace('_', "-");
                    if let Some(org) = self.organizations.iter_mut().find(|o| o.name == org_name) {
                        org.token = value;
                    } else {
                        let base_url_key = format!("GREPORT_ORG_{}_BASE_URL", org_upper);
                        self.organizations.push(OrgConfig {
                            name: org_name,
                            token: value,
                            base_url: std::env::var(base_url_key).ok(),
                            repos: None,
                        });
                    }
                }
            }
        }
    }

    /// Get GitHub token for a specific organization
    pub fn github_token_for_org(&self, org: &str) -> Option<String> {
        let org_lower = org.to_lowercase();
        self.organizations
            .iter()
            .find(|o| o.name.to_lowercase() == org_lower)
            .map(|o| o.token.clone())
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

    /// Collect repos from all configured organizations.
    ///
    /// Returns `RepoId` for each repo listed in each org's `repos` field.
    /// Repos are listed as short names (e.g. "my-repo") and expanded to
    /// `org-name/my-repo`.
    pub fn resolved_repos(&self) -> Vec<crate::client::RepoId> {
        self.organizations
            .iter()
            .flat_map(|org| {
                org.repos
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(move |repo| crate::client::RepoId::new(&org.name, repo))
            })
            .collect()
    }

    /// Collect repos for a single organization by name.
    ///
    /// Returns `RepoId` for each repo listed in that org's `repos` field.
    pub fn resolved_repos_for_org(&self, org: &str) -> Vec<crate::client::RepoId> {
        let org_lower = org.to_lowercase();
        self.organizations
            .iter()
            .filter(|o| o.name.to_lowercase() == org_lower)
            .flat_map(|o| {
                o.repos
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(move |repo| crate::client::RepoId::new(&o.name, repo))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse_multi_org() {
        let toml_str = r#"
[github]
token = "ghp_default"

[[organizations]]
name = "org-alpha"
token = "ghp_alpha"

[[organizations]]
name = "org-beta"
token = "ghp_beta"
base_url = "https://github.enterprise.com/api/v3"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.organizations.len(), 2);
        assert_eq!(config.organizations[0].name, "org-alpha");
        assert_eq!(config.organizations[0].token, "ghp_alpha");
        assert!(config.organizations[0].base_url.is_none());
        assert_eq!(config.organizations[1].name, "org-beta");
        assert_eq!(config.organizations[1].token, "ghp_beta");
        assert_eq!(
            config.organizations[1].base_url.as_deref(),
            Some("https://github.enterprise.com/api/v3")
        );
    }

    #[test]
    fn test_config_backward_compat() {
        let toml_str = r#"
[github]
token = "ghp_default"

[defaults]
format = "json"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.organizations.is_empty());
        assert_eq!(config.github.token.as_deref(), Some("ghp_default"));
        assert_eq!(config.defaults.format, "json");
    }

    #[test]
    fn test_config_org_token_lookup() {
        let config = Config {
            organizations: vec![
                OrgConfig {
                    name: "my-org".to_string(),
                    token: "ghp_myorg".to_string(),
                    base_url: None,
                    repos: None,
                },
                OrgConfig {
                    name: "other-org".to_string(),
                    token: "ghp_other".to_string(),
                    base_url: None,
                    repos: None,
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            config.github_token_for_org("my-org"),
            Some("ghp_myorg".to_string())
        );
        assert_eq!(
            config.github_token_for_org("other-org"),
            Some("ghp_other".to_string())
        );
        assert_eq!(config.github_token_for_org("unknown-org"), None);
    }

    #[test]
    fn test_config_org_token_case_insensitive() {
        let config = Config {
            organizations: vec![OrgConfig {
                name: "My-Org".to_string(),
                token: "ghp_myorg".to_string(),
                base_url: None,
                repos: None,
            }],
            ..Default::default()
        };

        assert_eq!(
            config.github_token_for_org("my-org"),
            Some("ghp_myorg".to_string())
        );
        assert_eq!(
            config.github_token_for_org("MY-ORG"),
            Some("ghp_myorg".to_string())
        );
        assert_eq!(
            config.github_token_for_org("My-Org"),
            Some("ghp_myorg".to_string())
        );
    }

    #[test]
    fn test_config_parse_org_repos() {
        let toml_str = r#"
[github]
token = "ghp_default"

[[organizations]]
name = "org-alpha"
token = "ghp_alpha"
repos = ["api-service", "web-frontend"]

[[organizations]]
name = "org-beta"
token = "ghp_beta"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.organizations.len(), 2);

        let alpha = &config.organizations[0];
        let alpha_repos = alpha.repos.as_ref().unwrap();
        assert_eq!(alpha_repos, &["api-service", "web-frontend"]);

        let beta = &config.organizations[1];
        assert!(beta.repos.is_none());
    }

    #[test]
    fn test_resolved_repos_for_org() {
        let config = Config {
            organizations: vec![
                OrgConfig {
                    name: "org-alpha".to_string(),
                    token: "ghp_alpha".to_string(),
                    base_url: None,
                    repos: Some(vec!["api".to_string(), "web".to_string()]),
                },
                OrgConfig {
                    name: "org-beta".to_string(),
                    token: "ghp_beta".to_string(),
                    base_url: None,
                    repos: Some(vec!["sdk".to_string()]),
                },
            ],
            ..Default::default()
        };

        let alpha_repos = config.resolved_repos_for_org("org-alpha");
        assert_eq!(alpha_repos.len(), 2);
        assert_eq!(alpha_repos[0].full_name(), "org-alpha/api");
        assert_eq!(alpha_repos[1].full_name(), "org-alpha/web");

        let beta_repos = config.resolved_repos_for_org("org-beta");
        assert_eq!(beta_repos.len(), 1);
        assert_eq!(beta_repos[0].full_name(), "org-beta/sdk");

        // Case-insensitive lookup
        let upper = config.resolved_repos_for_org("ORG-ALPHA");
        assert_eq!(upper.len(), 2);

        // Unknown org returns empty
        let unknown = config.resolved_repos_for_org("unknown");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_resolved_repos_all_orgs() {
        let config = Config {
            organizations: vec![
                OrgConfig {
                    name: "org-alpha".to_string(),
                    token: "ghp_alpha".to_string(),
                    base_url: None,
                    repos: Some(vec!["api".to_string()]),
                },
                OrgConfig {
                    name: "org-beta".to_string(),
                    token: "ghp_beta".to_string(),
                    base_url: None,
                    repos: Some(vec!["sdk".to_string(), "cli".to_string()]),
                },
            ],
            ..Default::default()
        };

        let all = config.resolved_repos();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].full_name(), "org-alpha/api");
        assert_eq!(all[1].full_name(), "org-beta/sdk");
        assert_eq!(all[2].full_name(), "org-beta/cli");
    }

    #[test]
    fn test_mask_token_long() {
        let masked = super::mask_token("ghp_abcdefghij1234");
        assert_eq!(masked, "ghp_****1234");
    }

    #[test]
    fn test_mask_token_short() {
        let masked = super::mask_token("ghp_abcd");
        assert_eq!(masked, "ghp_****");
    }

    #[test]
    fn test_mask_token_very_short() {
        let masked = super::mask_token("abc");
        assert_eq!(masked, "abc****");
    }

    #[test]
    fn test_resolved_repos_empty_when_no_repos() {
        let config = Config {
            organizations: vec![OrgConfig {
                name: "org-alpha".to_string(),
                token: "ghp_alpha".to_string(),
                base_url: None,
                repos: None,
            }],
            ..Default::default()
        };

        assert!(config.resolved_repos().is_empty());
        assert!(config.resolved_repos_for_org("org-alpha").is_empty());
    }
}
