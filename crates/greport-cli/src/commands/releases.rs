//! Release command handlers

use crate::args::{OutputFormat, ReleasesCommands};
use crate::output::Formatter;
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};
use greport_core::models::IssueState;
use greport_core::reports::ReleaseNotesGenerator;

pub async fn handle_releases(
    client: &impl GitHubClient,
    repo: &RepoId,
    command: ReleasesCommands,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let formatter = Formatter::new(format);

    match command {
        ReleasesCommands::List { limit } => {
            let releases = client.list_releases(repo).await?;
            let releases: Vec<_> = releases.into_iter().take(limit).collect();
            formatter.format_releases(&releases)?;
        }

        ReleasesCommands::Notes { milestone, version } => {
            // Get milestone
            let milestones = client.list_milestones(repo).await?;
            let ms = milestones
                .iter()
                .find(|m| m.title.eq_ignore_ascii_case(&milestone))
                .ok_or_else(|| anyhow::anyhow!("Milestone not found: {}", milestone))?;

            // Get closed issues for this milestone
            let issues = client.list_issues(repo, IssueParams::closed()).await?;
            let milestone_issues: Vec<_> = issues
                .into_iter()
                .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(ms.id))
                .filter(|i| i.state == IssueState::Closed)
                .collect();

            // Get merged PRs (simplified - would need milestone filter)
            let prs = client.list_pulls(repo, PullParams::all()).await?;
            let merged_prs: Vec<_> = prs.into_iter().filter(|p| p.merged).collect();

            // Generate release notes
            let generator = ReleaseNotesGenerator::with_defaults();
            let version_str = version.unwrap_or_else(|| milestone.clone());
            let notes = generator.generate(&version_str, &milestone_issues, &merged_prs);

            formatter.format_release_notes(&notes)?;
        }

        ReleasesCommands::Progress { milestone } => {
            let milestones = client.list_milestones(repo).await?;
            let ms = milestones
                .into_iter()
                .find(|m| m.title.eq_ignore_ascii_case(&milestone))
                .ok_or_else(|| anyhow::anyhow!("Milestone not found: {}", milestone))?;

            formatter.format_milestone_progress(&ms)?;
        }
    }

    Ok(())
}
