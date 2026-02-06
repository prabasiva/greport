//! greport CLI - GitHub reporting and analytics tool

mod args;
mod commands;
mod config;
mod output;

use args::{Cli, Commands};
use clap::Parser;
use greport_core::{OctocrabClient, RepoId};
use std::process::ExitCode;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// ASCII art banner for greport
const BANNER: &str = r#"
   ______                            __
  / ____/_______  ____  ____  _____/ /_
 / / __/ ___/ _ \/ __ \/ __ \/ ___/ __/
/ /_/ / /  /  __/ /_/ / /_/ / /  / /_
\____/_/   \___/ .___/\____/_/   \__/
              /_/

  GitHub Repository Analytics & Reporting Tool
  =============================================
"#;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            print_error(&e);
            ExitCode::FAILURE
        }
    }
}

/// Print error message in a user-friendly format
fn print_error(error: &anyhow::Error) {
    // Use Debug format for pattern matching (includes full error chain)
    let error_debug = format!("{:?}", error);
    // Use Display format for user-facing messages (clean, no backtrace)
    let error_display = format!("{}", error);

    if error_debug.contains("Resource not accessible by personal access token") {
        eprintln!("Error: Access denied - your token doesn't have permission for this resource.");
        eprintln!();
        eprintln!("Possible causes:");
        eprintln!("  - The repository may be private and your token lacks 'repo' scope");
        eprintln!("  - The resource (releases, issues, etc.) may not exist");
        eprintln!("  - Your token may have expired");
        eprintln!();
        eprintln!("Solutions:");
        eprintln!("  - Generate a new token with appropriate scopes at:");
        eprintln!("    https://github.com/settings/tokens");
        eprintln!("  - For private repos, ensure the token has 'repo' scope");
        eprintln!("  - For public repos, 'public_repo' scope is sufficient");
    } else if error_debug.contains("Not Found") || error_debug.contains("404") {
        eprintln!("Error: Resource not found.");
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - The repository name is correct (format: owner/repo)");
        eprintln!("  - The repository exists and is accessible");
        eprintln!("  - Your token has access to this repository");
    } else if error_debug.contains("Bad credentials") || error_debug.contains("401") {
        eprintln!("Error: Invalid GitHub token.");
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - Your token is correct and hasn't expired");
        eprintln!("  - The token is properly set in config.toml or GITHUB_TOKEN env var");
    } else if error_debug.contains("rate limit") || error_debug.contains("403") {
        eprintln!("Error: GitHub API rate limit exceeded.");
        eprintln!();
        eprintln!("Solutions:");
        eprintln!("  - Wait for the rate limit to reset");
        eprintln!("  - Use an authenticated token for higher limits");
        eprintln!("  - Check your rate limit status: greport rate-limit");
    } else if error_debug.contains("Missing GitHub token") {
        eprintln!("Error: Missing GitHub token.");
        eprintln!();
        eprintln!("Set your token using one of these methods:");
        eprintln!("  1. Environment variable: export GITHUB_TOKEN=ghp_xxx");
        eprintln!("  2. Config file: add 'token = \"ghp_xxx\"' to [github] section");
        eprintln!("     in ~/.config/greport/config.toml");
    } else if error_debug.contains("No repository specified") {
        eprintln!("Error: No repository specified.");
        eprintln!();
        eprintln!("Specify a repository using:");
        eprintln!("  1. Command line: greport -r owner/repo <command>");
        eprintln!("  2. Config file: add 'repo = \"owner/repo\"' to [defaults] section");
    } else if error_debug.contains("Invalid repository format") {
        // Extract the invalid repo name from the error message
        let repo_name = error_display
            .split("Invalid repository format: ")
            .nth(1)
            .and_then(|s| s.split('.').next())
            .unwrap_or("unknown");
        eprintln!("Error: Invalid repository format: '{}'", repo_name);
        eprintln!();
        eprintln!("Repository format should be: owner/repo");
        eprintln!("Example: greport -r microsoft/vscode issues list");
    } else if error_debug.contains("tcp connect error")
        || error_debug.contains("Connection refused")
        || error_debug.contains("timed out")
        || error_debug.contains("connect error")
        || error_debug.contains("client error (Connect)")
    {
        eprintln!("Error: Failed to connect to GitHub API.");
        eprintln!();
        eprintln!("Possible causes:");
        eprintln!("  - Network connectivity issues");
        eprintln!("  - GitHub API is unreachable");
        eprintln!("  - Firewall or proxy blocking the connection");
        if error_debug.contains("timed out") {
            eprintln!("  - Connection timed out (server may be slow or unreachable)");
        }
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - Your internet connection");
        eprintln!("  - The GitHub API base URL in your config (if using GitHub Enterprise)");
        eprintln!("  - Proxy/firewall settings");
    } else if error_debug.contains("dns error") || error_debug.contains("resolve") {
        eprintln!("Error: Failed to resolve GitHub API hostname.");
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - Your internet connection");
        eprintln!("  - DNS settings");
        eprintln!("  - The GitHub API base URL in your config (if using GitHub Enterprise)");
    } else {
        // Generic error message - use Display format (no backtrace)
        eprintln!("Error: {}", error_display);

        // Show cause chain if available (but not the full backtrace)
        let mut source = error.source();
        while let Some(cause) = source {
            let cause_str = cause.to_string();
            // Skip duplicate messages
            if !error_display.contains(&cause_str) {
                eprintln!("  Caused by: {}", cause_str);
            }
            source = cause.source();
        }
    }
}

/// Print the greport banner
fn print_banner() {
    eprintln!("{}", BANNER);
}

async fn run() -> anyhow::Result<()> {
    // Load .env file if present
    let dotenv_result = dotenvy::dotenv();

    // Load config early for log level resolution (env var > config.toml > default)
    let early_config = greport_core::Config::load(None).unwrap_or_default();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            early_config.rust_log("warn"),
        ))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // Log .env loading result
    match dotenv_result {
        Ok(path) => debug!("Loaded .env file from: {}", path.display()),
        Err(_) => debug!("No .env file found (this is normal)"),
    }

    // Print banner
    print_banner();

    // Parse arguments
    let cli = Cli::parse();
    debug!(repo = ?cli.repo, format = ?cli.format, "Parsed CLI arguments");

    // Load configuration
    let config_path = cli.config.as_deref();
    debug!(config_path = ?config_path, "Loading configuration");
    let cfg = config::load_config(config_path)?;

    // Log configuration details
    if let Some(ref path) = cli.config {
        info!(path = %path, "Configuration loaded from custom path");
    } else {
        let default_path = config::default_config_path()?;
        if default_path.exists() {
            info!(path = %default_path.display(), "Configuration loaded from default path");
        } else {
            debug!(path = %default_path.display(), "No config file found, using defaults");
        }
    }

    // Handle config command separately (doesn't need GitHub client)
    if let Commands::Config(args) = &cli.command {
        return commands::config::handle_config(&args.command);
    }

    // Resolve token source
    let (token, token_source) = if let Some(ref t) = cfg.github.token {
        debug!("Using GitHub token from config file");
        (t.clone(), "config file")
    } else if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        debug!("Using GitHub token from GITHUB_TOKEN environment variable");
        (t, "GITHUB_TOKEN env var")
    } else {
        anyhow::bail!("Missing GitHub token. Set GITHUB_TOKEN environment variable or configure in ~/.config/greport/config.toml");
    };

    // Resolve base_url source
    let (base_url, base_url_source) = if let Some(ref url) = cfg.github.base_url {
        debug!(url = %url, "Using base URL from config file");
        (Some(url.clone()), Some("config file"))
    } else if let Ok(url) = std::env::var("GITHUB_BASE_URL") {
        debug!(url = %url, "Using base URL from GITHUB_BASE_URL environment variable");
        (Some(url), Some("GITHUB_BASE_URL env var"))
    } else {
        debug!("No base URL configured, using default GitHub.com API");
        (None, None)
    };

    // Log connection info
    info!(
        token_source = token_source,
        base_url = base_url
            .as_deref()
            .unwrap_or("https://api.github.com (default)"),
        base_url_source = base_url_source.unwrap_or("default"),
        "Connecting to GitHub API"
    );

    let client = OctocrabClient::new(&token, base_url.as_deref())?;
    info!("GitHub client initialized successfully");

    // Resolve repository
    let (repo, repo_source) = match (&cli.repo, &cfg.defaults.repo) {
        (Some(r), _) => {
            debug!(repo = %r, "Using repository from command line argument");
            (RepoId::parse(r)?, "command line (-r/--repo)")
        }
        (None, Some(r)) => {
            debug!(repo = %r, "Using repository from config file defaults");
            (RepoId::parse(r)?, "config file (defaults.repo)")
        }
        (None, None) => {
            anyhow::bail!("No repository specified. Use -r/--repo or set defaults.repo in config");
        }
    };
    info!(
        repo = %repo,
        source = repo_source,
        "Target repository"
    );

    // Execute command
    match cli.command {
        Commands::Issues(args) => {
            commands::issues::handle_issues(&client, &repo, args.command, cli.format, &cfg).await?;
        }
        Commands::Prs(args) => {
            commands::pulls::handle_pulls(&client, &repo, args.command, cli.format).await?;
        }
        Commands::Releases(args) => {
            commands::releases::handle_releases(&client, &repo, args.command, cli.format).await?;
        }
        Commands::Contrib(args) => {
            commands::contrib::handle_contrib(&client, &repo, args.command, cli.format).await?;
        }
        Commands::Sync(args) => {
            commands::sync::handle_sync(&client, &repo, args).await?;
        }
        Commands::Config(_) => {
            // Already handled above
            unreachable!()
        }
    }

    Ok(())
}
