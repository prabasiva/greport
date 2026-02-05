//! JSON output formatting

use greport_core::models::{Issue, PullRequest, Release};
use serde::Serialize;

pub fn format_json<T: Serialize + ?Sized>(data: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

pub fn format_issues(issues: &[Issue]) -> anyhow::Result<()> {
    format_json(issues)
}

pub fn format_pulls(prs: &[PullRequest]) -> anyhow::Result<()> {
    format_json(prs)
}

pub fn format_releases(releases: &[Release]) -> anyhow::Result<()> {
    format_json(releases)
}
