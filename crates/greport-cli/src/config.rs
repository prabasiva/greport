//! CLI configuration handling

use greport_core::Config;
use std::path::PathBuf;

/// Load configuration from file
pub fn load_config(path: Option<&str>) -> anyhow::Result<Config> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => default_config_path()?,
    };

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

/// Get default configuration file path
pub fn default_config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home.join(".config").join("greport").join("config.toml"))
}

/// Create default configuration file
pub fn create_default_config(force: bool) -> anyhow::Result<PathBuf> {
    let path = default_config_path()?;

    if path.exists() && !force {
        anyhow::bail!(
            "Configuration file already exists at {}. Use --force to overwrite.",
            path.display()
        );
    }

    // Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let default_config = r#"# greport configuration

[github]
# GitHub personal access token (can also use GITHUB_TOKEN env var)
# token = "ghp_xxxx"

# GitHub Enterprise base URL (optional, for GitHub Enterprise Server)
# base_url = "https://github.mycompany.com/api/v3"

[defaults]
# Default repository (owner/repo)
# repo = "owner/repo"

# Default output format (table, json, csv, markdown)
format = "table"

# Cache TTL in seconds
cache_ttl = 3600

[sla]
# Default response time SLA (hours)
response_time_hours = 24

# Default resolution time SLA (hours)
resolution_time_hours = 168

[sla.priority.critical]
response_time_hours = 4
resolution_time_hours = 24

[sla.priority.high]
response_time_hours = 8
resolution_time_hours = 72

[database]
# PostgreSQL connection URL (can also use DATABASE_URL env var)
# url = "postgres://user:password@localhost:5432/greport"

# Maximum connections in pool (default: 10)
# max_connections = 10

# Connection acquire timeout in seconds (default: 5)
# acquire_timeout_secs = 5

# Run migrations on startup (default: true)
# run_migrations = true

[server]
# Bind host address (can also use API_HOST env var, default: 0.0.0.0)
# host = "0.0.0.0"

# Bind port (can also use API_PORT env var, default: 9423)
# port = 9423

# Rate limit: max requests per minute per client (default: 60)
# rate_limit_per_minute = 60

# Cache TTL in seconds for API responses (default: 300)
# cache_ttl_seconds = 300

# Maximum page size for paginated responses (default: 100)
# max_page_size = 100

# Require API key authentication (default: false)
# require_auth = false

[logging]
# Log level filter (can also use RUST_LOG env var)
# Examples: "info", "debug", "info,tower_http=debug"
# level = "info"
"#;

    std::fs::write(&path, default_config)?;

    Ok(path)
}
