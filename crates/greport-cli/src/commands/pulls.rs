//! Pull request command handlers

use crate::args::{OutputFormat, PrsCommands};
use crate::output::Formatter;
use greport_core::client::{GitHubClient, PullParams, RepoId};
use greport_core::metrics::PullMetricsCalculator;

pub async fn handle_pulls(
    client: &impl GitHubClient,
    repo: &RepoId,
    command: PrsCommands,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let formatter = Formatter::new(format);

    match command {
        PrsCommands::List { state, author, limit } => {
            let params = PullParams {
                state: state.into(),
                per_page: limit.min(100),
                ..Default::default()
            };

            let mut prs = client.list_pulls(repo, params).await?;

            // Filter by author if specified
            if let Some(author) = author {
                prs.retain(|pr| pr.author.login.eq_ignore_ascii_case(&author));
            }

            let prs: Vec<_> = prs.into_iter().take(limit).collect();
            formatter.format_pulls(&prs)?;
        }

        PrsCommands::Metrics => {
            let prs = client.list_pulls(repo, PullParams::all()).await?;
            let metrics = PullMetricsCalculator::calculate(&prs);
            formatter.format_pull_metrics(&metrics)?;
        }

        PrsCommands::Unreviewed => {
            let prs = client.list_pulls(repo, PullParams::open()).await?;

            // Filter for PRs that are ready for review (not drafts)
            let unreviewed: Vec<_> = prs
                .into_iter()
                .filter(|pr| pr.is_ready_for_review())
                .collect();

            if unreviewed.is_empty() {
                println!("No unreviewed pull requests found.");
            } else {
                println!("Found {} PRs awaiting review:\n", unreviewed.len());
                formatter.format_pulls(&unreviewed)?;
            }
        }
    }

    Ok(())
}
