//! Contributor command handlers

use crate::args::{ContribCommands, ContribSort, OutputFormat};
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};
use std::collections::HashMap;

pub async fn handle_contrib(
    client: &impl GitHubClient,
    repo: &RepoId,
    command: ContribCommands,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    match command {
        ContribCommands::List { sort_by, limit } => {
            // Get all issues and PRs
            let issues = client.list_issues(repo, IssueParams::all()).await?;
            let prs = client.list_pulls(repo, PullParams::all()).await?;

            // Aggregate stats per contributor
            let mut contributors: HashMap<String, ContribStats> = HashMap::new();

            for issue in &issues {
                let entry = contributors
                    .entry(issue.author.login.clone())
                    .or_default();
                entry.issues_created += 1;
            }

            for pr in &prs {
                let entry = contributors
                    .entry(pr.author.login.clone())
                    .or_default();
                entry.prs_created += 1;
                if pr.merged {
                    entry.prs_merged += 1;
                }
            }

            // Sort by specified metric
            let mut sorted: Vec<_> = contributors.into_iter().collect();
            match sort_by {
                ContribSort::Issues => sorted.sort_by(|a, b| b.1.issues_created.cmp(&a.1.issues_created)),
                ContribSort::Prs => sorted.sort_by(|a, b| b.1.prs_created.cmp(&a.1.prs_created)),
                ContribSort::Comments => sorted.sort_by(|a, b| b.1.issues_created.cmp(&a.1.issues_created)),
            }

            // Print results
            println!("Contributors (top {}):\n", limit);
            println!("{:<20} {:>8} {:>8} {:>8}", "User", "Issues", "PRs", "Merged");
            println!("{}", "-".repeat(48));

            for (user, stats) in sorted.into_iter().take(limit) {
                println!(
                    "{:<20} {:>8} {:>8} {:>8}",
                    user, stats.issues_created, stats.prs_created, stats.prs_merged
                );
            }
        }

        ContribCommands::Stats { username } => {
            let issues = client.list_issues(repo, IssueParams::all()).await?;
            let prs = client.list_pulls(repo, PullParams::all()).await?;

            let user_issues: Vec<_> = issues
                .iter()
                .filter(|i| i.author.login.eq_ignore_ascii_case(&username))
                .collect();

            let user_prs: Vec<_> = prs
                .iter()
                .filter(|p| p.author.login.eq_ignore_ascii_case(&username))
                .collect();

            let assigned_issues: Vec<_> = issues
                .iter()
                .filter(|i| i.assignees.iter().any(|a| a.login.eq_ignore_ascii_case(&username)))
                .collect();

            println!("Contributor Stats: {}\n", username);
            println!("Issues created: {}", user_issues.len());
            println!("Issues assigned: {}", assigned_issues.len());
            println!("PRs created: {}", user_prs.len());
            println!("PRs merged: {}", user_prs.iter().filter(|p| p.merged).count());

            let total_additions: u32 = user_prs.iter().map(|p| p.additions).sum();
            let total_deletions: u32 = user_prs.iter().map(|p| p.deletions).sum();
            println!("\nLines added: {}", total_additions);
            println!("Lines deleted: {}", total_deletions);
        }
    }

    Ok(())
}

#[derive(Default)]
struct ContribStats {
    issues_created: usize,
    prs_created: usize,
    prs_merged: usize,
}
