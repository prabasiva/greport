//! Organization management commands

use crate::args::OrgsCommands;
use comfy_table::{Cell, Table};
use greport_core::config::{mask_token, Config};

/// Handle orgs subcommands.
pub fn handle_orgs(command: &OrgsCommands, config: &Config) -> anyhow::Result<()> {
    match command {
        OrgsCommands::List => list_orgs(config),
        OrgsCommands::Show { name } => show_org(config, name),
    }
}

fn list_orgs(config: &Config) -> anyhow::Result<()> {
    if config.organizations.is_empty() {
        println!("No organizations configured.");
        println!();
        println!("Add organizations to your config.toml:");
        println!("  [[organizations]]");
        println!("  name = \"my-org\"");
        println!("  token = \"ghp_xxx\"");
        println!("  repos = [\"repo1\", \"repo2\"]");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Name"),
        Cell::new("Base URL"),
        Cell::new("Repos"),
    ]);

    for org in &config.organizations {
        let base_url = org.base_url.as_deref().unwrap_or("https://api.github.com");
        let repo_count = org.repos.as_deref().map_or(0, |r| r.len());
        table.add_row(vec![
            Cell::new(&org.name),
            Cell::new(base_url),
            Cell::new(repo_count),
        ]);
    }

    println!("{table}");
    println!();
    println!("Total: {} organization(s)", config.organizations.len());
    Ok(())
}

fn show_org(config: &Config, name: &str) -> anyhow::Result<()> {
    let name_lower = name.to_lowercase();
    let org = config
        .organizations
        .iter()
        .find(|o| o.name.to_lowercase() == name_lower);

    match org {
        Some(org) => {
            println!("Organization: {}", org.name);
            println!("Token:        {}", mask_token(&org.token));
            println!(
                "Base URL:     {}",
                org.base_url.as_deref().unwrap_or("https://api.github.com")
            );

            match &org.repos {
                Some(repos) if !repos.is_empty() => {
                    println!("Repos ({}):", repos.len());
                    for repo in repos {
                        println!("  - {}/{}", org.name, repo);
                    }
                }
                _ => {
                    println!("Repos:        (none configured)");
                }
            }
            Ok(())
        }
        None => {
            anyhow::bail!(
                "Organization '{}' not found in configuration.\nRun 'greport orgs list' to see configured organizations.",
                name
            );
        }
    }
}
