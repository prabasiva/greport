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
"#;

    std::fs::write(&path, default_config)?;

    Ok(path)
}
