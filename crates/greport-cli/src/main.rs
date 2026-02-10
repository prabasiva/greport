//! greport CLI - GitHub reporting and analytics tool

mod args;
mod commands;
mod config;
mod output;

use args::{Cli, Commands};
use clap::Parser;
use greport_core::{Config, GitHubClientRegistry, OctocrabClient, RepoId};
use std::process::ExitCode;
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Whether the user targets a single repo or multiple repos.
enum RepoTarget {
    Single(RepoId),
    Multi(Vec<RepoId>),
}

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
        eprintln!(
            "  3. Config file: add 'repos = [\"repo1\", \"repo2\"]' to [[organizations]] entries"
        );
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

    // Parse arguments first so --config flag is available for log level
    let cli = Cli::parse();

    // Load config from the correct path (--config flag or default)
    let config_path = cli.config.as_deref();
    let cfg = config::load_config(config_path)?;

    // Initialize logging using the resolved config (env var > config.toml > "warn")
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(cfg.rust_log("warn")))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // Log .env loading result
    match dotenv_result {
        Ok(path) => debug!("Loaded .env file from: {}", path.display()),
        Err(_) => debug!("No .env file found (this is normal)"),
    }

    // Print banner
    print_banner();

    debug!(repo = ?cli.repo, format = ?cli.format, "Parsed CLI arguments");

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

    // Handle orgs command separately (doesn't need GitHub client)
    if let Commands::Orgs(args) = &cli.command {
        return commands::orgs::handle_orgs(&args.command, &cfg);
    }

    // Build client registry (supports multi-org and single-token configs)
    let has_orgs = !cfg.organizations.is_empty();
    let registry = if has_orgs {
        info!(
            org_count = cfg.organizations.len(),
            "Building multi-org client registry"
        );
        GitHubClientRegistry::from_config(&cfg)?
    } else {
        // Legacy single-token path: resolve token from config or env
        let (token, token_source) = if let Some(ref t) = cfg.github.token {
            debug!("Using GitHub token from config file");
            (t.clone(), "config file")
        } else if let Ok(t) = std::env::var("GITHUB_TOKEN") {
            debug!("Using GitHub token from GITHUB_TOKEN environment variable");
            (t, "GITHUB_TOKEN env var")
        } else {
            anyhow::bail!(
                "Missing GitHub token.\n\n\
                 Set your token using one of these methods:\n  \
                 1. Environment variable: export GITHUB_TOKEN=ghp_xxx\n  \
                 2. Config file: add 'token = \"ghp_xxx\"' to [github] section\n     \
                    in ~/.config/greport/config.toml\n  \
                 3. Multi-org: add [[organizations]] entries with per-org tokens"
            );
        };

        let base_url = cfg
            .github
            .base_url
            .clone()
            .or_else(|| std::env::var("GITHUB_BASE_URL").ok());

        info!(
            token_source = token_source,
            base_url = base_url
                .as_deref()
                .unwrap_or("https://api.github.com (default)"),
            "Connecting to GitHub API"
        );

        // Build a registry with just the default client
        let client = OctocrabClient::new(&token, base_url.as_deref())?;
        // Wrap in a minimal registry so all code paths use the same type
        GitHubClientRegistry::with_default(client)
    };
    info!("GitHub client initialized successfully");

    // Validate tokens when verbose mode is enabled
    if cli.verbose {
        let valid = registry.validate_tokens().await;
        info!(valid_tokens = valid, "Token validation complete");
    }

    // Resolve repository target using precedence rules:
    // 1. -r org/repo  -> Single repo (highest priority)
    // 2. --org <name> without -r -> Multi: that org's configured repos
    // 3. No -r, no --org -> Multi: all orgs' configured repos
    // 4. No -r, no org repos -> Single: defaults.repo fallback
    // 5. None of the above -> error
    let target = if let Some(ref r) = cli.repo {
        debug!(repo = %r, "Using repository from command line argument");
        RepoTarget::Single(RepoId::parse(r)?)
    } else if let Some(ref org_name) = cli.org {
        let repos = cfg.resolved_repos_for_org(org_name);
        if repos.is_empty() {
            anyhow::bail!(
                "No repos configured for organization '{}'. \
                 Add repos = [\"repo1\", \"repo2\"] to the [[organizations]] entry, \
                 or use -r org/repo to specify a single repo.",
                org_name
            );
        }
        debug!(org = %org_name, count = repos.len(), "Using repos from org config");
        RepoTarget::Multi(repos)
    } else {
        let all_repos = cfg.resolved_repos();
        if !all_repos.is_empty() {
            debug!(count = all_repos.len(), "Using repos from all org configs");
            RepoTarget::Multi(all_repos)
        } else if let Some(ref r) = cfg.defaults.repo {
            debug!(repo = %r, "Using repository from config file defaults");
            RepoTarget::Single(RepoId::parse(r)?)
        } else {
            anyhow::bail!("No repository specified. Use -r/--repo or set defaults.repo in config");
        }
    };

    // Execute across target repo(s)
    match target {
        RepoTarget::Single(repo) => {
            info!(repo = %repo, "Target repository");
            let client = registry.client_for_repo(&repo)?;
            execute_command(client.clone(), &repo, &cli.command, cli.format, &cfg).await?;
        }
        RepoTarget::Multi(repos) => {
            let total = repos.len();
            info!(count = total, "Running across multiple repositories");
            let mut had_error = false;
            for (i, repo) in repos.iter().enumerate() {
                eprintln!("{}", format_repo_header(&repo.full_name(), i + 1, total));
                let client = match registry.client_for_repo(repo) {
                    Ok(c) => c.clone(),
                    Err(e) => {
                        eprintln!("  Error for {}: {}", repo, e);
                        eprintln!();
                        had_error = true;
                        continue;
                    }
                };
                if let Err(e) = execute_command(client, repo, &cli.command, cli.format, &cfg).await
                {
                    eprintln!("  Error for {}: {}", repo, e);
                    eprintln!();
                    had_error = true;
                }
            }
            if had_error {
                anyhow::bail!("One or more repositories encountered errors");
            }
        }
    }

    Ok(())
}

/// Execute a single command against one repository.
async fn execute_command(
    client: Arc<OctocrabClient>,
    repo: &RepoId,
    command: &Commands,
    format: args::OutputFormat,
    cfg: &Config,
) -> anyhow::Result<()> {
    match command {
        Commands::Issues(args) => {
            commands::issues::handle_issues(
                client.as_ref(),
                repo,
                args.command.clone(),
                format,
                cfg,
            )
            .await?;
        }
        Commands::Prs(args) => {
            commands::pulls::handle_pulls(client.as_ref(), repo, args.command.clone(), format)
                .await?;
        }
        Commands::Releases(args) => {
            commands::releases::handle_releases(
                client.as_ref(),
                repo,
                args.command.clone(),
                format,
            )
            .await?;
        }
        Commands::Contrib(args) => {
            commands::contrib::handle_contrib(client.as_ref(), repo, args.command.clone(), format)
                .await?;
        }
        Commands::Sync(args) => {
            commands::sync::handle_sync(client.as_ref(), repo, args.clone()).await?;
        }
        Commands::Config(_) | Commands::Orgs(_) => {
            unreachable!()
        }
    }
    Ok(())
}

/// Format a header line for multi-repo output.
fn format_repo_header(repo_name: &str, index: usize, total: usize) -> String {
    let prefix = format!("--- {} ({}/{}) ", repo_name, index, total);
    let padding = if prefix.len() < 60 {
        "-".repeat(60 - prefix.len())
    } else {
        "---".to_string()
    };
    format!("{}{}", prefix, padding)
}
