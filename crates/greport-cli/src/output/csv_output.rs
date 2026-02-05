//! CSV output formatting

use greport_core::metrics::{IssueMetrics, PullMetrics, SlaReport, VelocityMetrics};
use greport_core::models::{Issue, PullRequest, Release};
use greport_core::reports::BurndownReport;
use std::io;

pub fn format_issues(issues: &[Issue]) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["number", "title", "state", "labels", "assignees", "age_days", "created_at"])?;

    for issue in issues {
        let labels = issue.label_names().join(";");
        let assignees = issue.assignee_logins().join(";");
        let state = match issue.state {
            greport_core::models::IssueState::Open => "open",
            greport_core::models::IssueState::Closed => "closed",
        };

        wtr.write_record([
            &issue.number.to_string(),
            &issue.title,
            state,
            &labels,
            &assignees,
            &issue.age_days().to_string(),
            &issue.created_at.to_rfc3339(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_issue_metrics(metrics: &IssueMetrics) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["metric", "value"])?;
    wtr.write_record(["total", &metrics.total.to_string()])?;
    wtr.write_record(["open", &metrics.open.to_string()])?;
    wtr.write_record(["closed", &metrics.closed.to_string()])?;
    wtr.write_record(["stale_count", &metrics.stale_count.to_string()])?;

    if let Some(avg) = metrics.avg_time_to_close_hours {
        wtr.write_record(["avg_time_to_close_hours", &format!("{:.2}", avg)])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_velocity(velocity: &VelocityMetrics) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["period_start", "opened", "closed", "net_change", "cumulative_open"])?;

    for dp in &velocity.data_points {
        wtr.write_record([
            &dp.period_start.to_rfc3339(),
            &dp.opened.to_string(),
            &dp.closed.to_string(),
            &dp.net_change.to_string(),
            &dp.cumulative_open.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_burndown(burndown: &BurndownReport) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["date", "remaining", "completed"])?;

    for dp in &burndown.data_points {
        wtr.write_record([
            &dp.date.format("%Y-%m-%d").to_string(),
            &dp.remaining.to_string(),
            &dp.completed.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_sla(sla: &SlaReport) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["issue_number", "title", "violation_type", "sla_hours", "actual_hours"])?;

    for v in &sla.violations {
        wtr.write_record([
            &v.issue_number.to_string(),
            &v.issue_title,
            &format!("{:?}", v.violation_type),
            &v.sla_hours.to_string(),
            &v.actual_hours.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_pulls(prs: &[PullRequest]) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["number", "title", "state", "author", "merged", "additions", "deletions"])?;

    for pr in prs {
        let state = match pr.state {
            greport_core::models::PullState::Open => "open",
            greport_core::models::PullState::Closed => "closed",
        };

        wtr.write_record([
            &pr.number.to_string(),
            &pr.title,
            state,
            &pr.author.login,
            &pr.merged.to_string(),
            &pr.additions.to_string(),
            &pr.deletions.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_pull_metrics(metrics: &PullMetrics) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["metric", "value"])?;
    wtr.write_record(["total", &metrics.total.to_string()])?;
    wtr.write_record(["open", &metrics.open.to_string()])?;
    wtr.write_record(["merged", &metrics.merged.to_string()])?;
    wtr.write_record(["closed_unmerged", &metrics.closed_unmerged.to_string()])?;

    if let Some(avg) = metrics.avg_time_to_merge_hours {
        wtr.write_record(["avg_time_to_merge_hours", &format!("{:.2}", avg)])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn format_releases(releases: &[Release]) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["tag_name", "name", "draft", "prerelease", "published_at"])?;

    for r in releases {
        wtr.write_record([
            &r.tag_name,
            r.name.as_deref().unwrap_or(""),
            &r.draft.to_string(),
            &r.prerelease.to_string(),
            &r.published_at
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
