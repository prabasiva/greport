//! greport CLI - GitHub reporting and analytics tool

mod args;
mod commands;
mod config;
mod output;

use args::{Cli, Commands};
use clap::Parser;
use greport_core::{OctocrabClient, RepoId};
use std::process::ExitCode;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    // Build full error string including cause chain for pattern matching
    let error_str = format!("{:?}", error);

    if error_str.contains("Resource not accessible by personal access token") {
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
    } else if error_str.contains("Not Found") || error_str.contains("404") {
        eprintln!("Error: Resource not found.");
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - The repository name is correct (format: owner/repo)");
        eprintln!("  - The repository exists and is accessible");
        eprintln!("  - Your token has access to this repository");
    } else if error_str.contains("Bad credentials") || error_str.contains("401") {
        eprintln!("Error: Invalid GitHub token.");
        eprintln!();
        eprintln!("Please check:");
        eprintln!("  - Your token is correct and hasn't expired");
        eprintln!("  - The token is properly set in config.toml or GITHUB_TOKEN env var");
    } else if error_str.contains("rate limit") || error_str.contains("403") {
        eprintln!("Error: GitHub API rate limit exceeded.");
        eprintln!();
        eprintln!("Solutions:");
        eprintln!("  - Wait for the rate limit to reset");
        eprintln!("  - Use an authenticated token for higher limits");
        eprintln!("  - Check your rate limit status: greport rate-limit");
    } else if error_str.contains("Missing GitHub token") {
        eprintln!("Error: {}", error_str);
        eprintln!();
        eprintln!("Set your token using one of these methods:");
        eprintln!("  1. Environment variable: export GITHUB_TOKEN=ghp_xxx");
        eprintln!("  2. Config file: add 'token = \"ghp_xxx\"' to [github] section");
        eprintln!("     in ~/.config/greport/config.toml");
    } else if error_str.contains("No repository specified") {
        eprintln!("Error: {}", error_str);
        eprintln!();
        eprintln!("Specify a repository using:");
        eprintln!("  1. Command line: greport -r owner/repo <command>");
        eprintln!("  2. Config file: add 'repo = \"owner/repo\"' to [defaults] section");
    } else if error_str.contains("Invalid repository format") {
        eprintln!("Error: {}", error_str);
        eprintln!();
        eprintln!("Repository format should be: owner/repo");
        eprintln!("Example: greport -r microsoft/vscode issues list");
    } else {
        // Generic error message
        eprintln!("Error: {}", error_str);

        // Show cause chain if available (but not the full backtrace)
        let mut source = error.source();
        while let Some(cause) = source {
            let cause_str = cause.to_string();
            // Skip duplicate messages and internal details
            if !error_str.contains(&cause_str) && !cause_str.contains("backtrace") {
                eprintln!("  Caused by: {}", cause_str);
            }
            source = cause.source();
        }
    }
}

async fn run() -> anyhow::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // Parse arguments
    let cli = Cli::parse();

    // Load configuration
    let cfg = config::load_config(cli.config.as_deref())?;

    // Handle config command separately (doesn't need GitHub client)
    if let Commands::Config(args) = &cli.command {
        return commands::config::handle_config(&args.command);
    }

    // Create GitHub client
    let token = cfg
        .github
        .token
        .clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| anyhow::anyhow!("Missing GitHub token. Set GITHUB_TOKEN environment variable or configure in ~/.config/greport/config.toml"))?;

    let client = OctocrabClient::new(&token)?;

    // Resolve repository
    let repo = match (&cli.repo, &cfg.defaults.repo) {
        (Some(r), _) => RepoId::parse(r)?,
        (None, Some(r)) => RepoId::parse(r)?,
        (None, None) => {
            anyhow::bail!("No repository specified. Use -r/--repo or set defaults.repo in config");
        }
    };

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
