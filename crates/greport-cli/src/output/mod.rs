//! Output formatting for CLI

mod csv_output;
mod json_output;
mod markdown_output;
mod table_output;

use crate::args::OutputFormat;
use greport_core::metrics::{IssueMetrics, PullMetrics, SlaReport, VelocityMetrics};
use greport_core::models::{Issue, Milestone, PullRequest, Release};
use greport_core::reports::{BurndownReport, ReleaseNotes};

/// Unified formatter for CLI output
pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    /// Create a new formatter
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Format and print issues
    pub fn format_issues(&self, issues: &[Issue]) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_issues(issues),
            OutputFormat::Csv => csv_output::format_issues(issues),
            OutputFormat::Markdown => markdown_output::format_issues(issues),
            OutputFormat::Table => table_output::format_issues(issues),
        }
    }

    /// Format and print issue metrics
    pub fn format_issue_metrics(&self, metrics: &IssueMetrics) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(metrics),
            OutputFormat::Csv => csv_output::format_issue_metrics(metrics),
            OutputFormat::Markdown => markdown_output::format_issue_metrics(metrics),
            OutputFormat::Table => table_output::format_issue_metrics(metrics),
        }
    }

    /// Format and print velocity metrics
    pub fn format_velocity(&self, velocity: &VelocityMetrics) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(velocity),
            OutputFormat::Csv => csv_output::format_velocity(velocity),
            OutputFormat::Markdown => markdown_output::format_velocity(velocity),
            OutputFormat::Table => table_output::format_velocity(velocity),
        }
    }

    /// Format and print burndown report
    pub fn format_burndown(&self, burndown: &BurndownReport) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(burndown),
            OutputFormat::Csv => csv_output::format_burndown(burndown),
            OutputFormat::Markdown => markdown_output::format_burndown(burndown),
            OutputFormat::Table => table_output::format_burndown(burndown),
        }
    }

    /// Format and print SLA report
    pub fn format_sla(&self, sla: &SlaReport) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(sla),
            OutputFormat::Csv => csv_output::format_sla(sla),
            OutputFormat::Markdown => markdown_output::format_sla(sla),
            OutputFormat::Table => table_output::format_sla(sla),
        }
    }

    /// Format and print pull requests
    pub fn format_pulls(&self, prs: &[PullRequest]) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_pulls(prs),
            OutputFormat::Csv => csv_output::format_pulls(prs),
            OutputFormat::Markdown => markdown_output::format_pulls(prs),
            OutputFormat::Table => table_output::format_pulls(prs),
        }
    }

    /// Format and print PR metrics
    pub fn format_pull_metrics(&self, metrics: &PullMetrics) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(metrics),
            OutputFormat::Csv => csv_output::format_pull_metrics(metrics),
            OutputFormat::Markdown => markdown_output::format_pull_metrics(metrics),
            OutputFormat::Table => table_output::format_pull_metrics(metrics),
        }
    }

    /// Format and print releases
    pub fn format_releases(&self, releases: &[Release]) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_releases(releases),
            OutputFormat::Csv => csv_output::format_releases(releases),
            OutputFormat::Markdown => markdown_output::format_releases(releases),
            OutputFormat::Table => table_output::format_releases(releases),
        }
    }

    /// Format and print release notes
    pub fn format_release_notes(&self, notes: &ReleaseNotes) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(notes),
            OutputFormat::Markdown => markdown_output::format_release_notes(notes),
            _ => markdown_output::format_release_notes(notes), // Default to markdown
        }
    }

    /// Format and print milestone progress
    pub fn format_milestone_progress(&self, milestone: &Milestone) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json_output::format_json(milestone),
            OutputFormat::Markdown => markdown_output::format_milestone_progress(milestone),
            OutputFormat::Table => table_output::format_milestone_progress(milestone),
            _ => table_output::format_milestone_progress(milestone),
        }
    }
}
