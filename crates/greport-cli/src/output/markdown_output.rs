//! Markdown output formatting

use greport_core::metrics::{IssueMetrics, PullMetrics, SlaReport, VelocityMetrics};
use greport_core::models::{Issue, IssueState, Milestone, PullRequest, PullState, Release};
use greport_core::reports::{BurndownReport, ReleaseNotes, ReleaseNotesGenerator};

pub fn format_issues(issues: &[Issue]) -> anyhow::Result<()> {
    println!("# Issues\n");
    println!("| # | Title | State | Labels | Assignee | Age |");
    println!("|---|-------|-------|--------|----------|-----|");

    for issue in issues {
        let state = match issue.state {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
        };
        let labels = issue.label_names().join(", ");
        let assignee = issue
            .assignees
            .first()
            .map(|a| a.login.as_str())
            .unwrap_or("-");

        println!(
            "| {} | {} | {} | {} | {} | {}d |",
            issue.number,
            issue.title.replace('|', "\\|"),
            state,
            labels,
            assignee,
            issue.age_days()
        );
    }

    println!("\n**Total:** {} issues", issues.len());
    Ok(())
}

pub fn format_issue_metrics(metrics: &IssueMetrics) -> anyhow::Result<()> {
    println!("# Issue Metrics\n");
    println!("## Summary\n");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Total | {} |", metrics.total);
    println!("| Open | {} |", metrics.open);
    println!("| Closed | {} |", metrics.closed);
    println!("| Stale | {} |", metrics.stale_count);

    if let Some(avg) = metrics.avg_time_to_close_hours {
        println!("| Avg Time to Close | {:.1}h ({:.1}d) |", avg, avg / 24.0);
    }

    println!("\n## By Label\n");
    println!("| Label | Count |");
    println!("|-------|-------|");
    let mut labels: Vec<_> = metrics.by_label.iter().collect();
    labels.sort_by(|a, b| b.1.cmp(a.1));
    for (label, count) in labels.iter().take(10) {
        println!("| {} | {} |", label, count);
    }

    println!("\n## Age Distribution\n");
    println!("| Age | Count |");
    println!("|-----|-------|");
    for bucket in &metrics.age_distribution.buckets {
        println!("| {} | {} |", bucket.label, bucket.count);
    }

    Ok(())
}

pub fn format_velocity(velocity: &VelocityMetrics) -> anyhow::Result<()> {
    println!("# Velocity Report\n");
    println!("**Period:** {}", velocity.period.label());
    println!("**Trend:** {}\n", velocity.trend.label());

    println!("| Period | Opened | Closed | Net | Total Open |");
    println!("|--------|--------|--------|-----|------------|");

    for dp in &velocity.data_points {
        println!(
            "| {} | {} | {} | {} | {} |",
            dp.period_start.format("%Y-%m-%d"),
            dp.opened,
            dp.closed,
            dp.net_change,
            dp.cumulative_open
        );
    }

    println!("\n**Averages:**");
    println!(
        "- Opened per {}: {:.1}",
        velocity.period.label(),
        velocity.avg_opened
    );
    println!(
        "- Closed per {}: {:.1}",
        velocity.period.label(),
        velocity.avg_closed
    );

    Ok(())
}

pub fn format_burndown(burndown: &BurndownReport) -> anyhow::Result<()> {
    println!("# Burndown: {}\n", burndown.milestone);
    println!("- **Total Issues:** {}", burndown.total_issues);
    println!(
        "- **Start Date:** {}",
        burndown.start_date.format("%Y-%m-%d")
    );

    if let Some(end) = burndown.end_date {
        println!("- **Due Date:** {}", end.format("%Y-%m-%d"));
    }

    if let Some(projected) = burndown.projected_completion {
        println!(
            "- **Projected Completion:** {}",
            projected.format("%Y-%m-%d")
        );
    }

    if let Some(last) = burndown.data_points.last() {
        let pct = if burndown.total_issues > 0 {
            (last.completed * 100) / burndown.total_issues
        } else {
            0
        };
        println!(
            "\n**Progress:** {}% ({}/{})",
            pct, last.completed, burndown.total_issues
        );
    }

    Ok(())
}

pub fn format_sla(sla: &SlaReport) -> anyhow::Result<()> {
    println!("# SLA Compliance Report\n");

    println!("## Summary\n");
    println!("| Metric | Met | Breached | Compliance |");
    println!("|--------|-----|----------|------------|");
    println!(
        "| Response | {} | {} | {:.1}% |",
        sla.response_sla_met, sla.response_sla_breached, sla.response_compliance_percent
    );
    println!(
        "| Resolution | {} | {} | {:.1}% |",
        sla.resolution_sla_met, sla.resolution_sla_breached, sla.resolution_compliance_percent
    );

    if !sla.violations.is_empty() {
        println!("\n## Violations\n");
        println!("| Issue | Type | SLA | Actual | Exceeded |");
        println!("|-------|------|-----|--------|----------|");

        for v in &sla.violations {
            println!(
                "| #{} | {:?} | {}h | {}h | {}h |",
                v.issue_number, v.violation_type, v.sla_hours, v.actual_hours, v.exceeded_by_hours
            );
        }
    }

    Ok(())
}

pub fn format_pulls(prs: &[PullRequest]) -> anyhow::Result<()> {
    println!("# Pull Requests\n");
    println!("| # | Title | State | Author | Size |");
    println!("|---|-------|-------|--------|------|");

    for pr in prs {
        let state = if pr.merged {
            "merged"
        } else {
            match pr.state {
                PullState::Open => "open",
                PullState::Closed => "closed",
            }
        };

        println!(
            "| {} | {} | {} | {} | {} |",
            pr.number,
            pr.title.replace('|', "\\|"),
            state,
            pr.author.login,
            pr.size_category().label()
        );
    }

    println!("\n**Total:** {} pull requests", prs.len());
    Ok(())
}

pub fn format_pull_metrics(metrics: &PullMetrics) -> anyhow::Result<()> {
    println!("# Pull Request Metrics\n");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Total | {} |", metrics.total);
    println!("| Open | {} |", metrics.open);
    println!("| Merged | {} |", metrics.merged);
    println!("| Closed (unmerged) | {} |", metrics.closed_unmerged);
    println!("| Drafts | {} |", metrics.draft_count);

    if let Some(avg) = metrics.avg_time_to_merge_hours {
        println!("| Avg Time to Merge | {:.1}h |", avg);
    }

    Ok(())
}

pub fn format_releases(releases: &[Release]) -> anyhow::Result<()> {
    println!("# Releases\n");
    println!("| Tag | Name | Type | Date |");
    println!("|-----|------|------|------|");

    for r in releases {
        let release_type = if r.draft {
            "draft"
        } else if r.prerelease {
            "prerelease"
        } else {
            "release"
        };

        let date = r
            .published_at
            .unwrap_or(r.created_at)
            .format("%Y-%m-%d")
            .to_string();

        println!(
            "| {} | {} | {} | {} |",
            r.tag_name,
            r.name.as_deref().unwrap_or("-"),
            release_type,
            date
        );
    }

    Ok(())
}

pub fn format_release_notes(notes: &ReleaseNotes) -> anyhow::Result<()> {
    let generator = ReleaseNotesGenerator::with_defaults();
    let md = generator.to_markdown(notes);
    println!("{}", md);
    Ok(())
}

pub fn format_milestone_progress(milestone: &Milestone) -> anyhow::Result<()> {
    println!("# Milestone: {}\n", milestone.title);

    if let Some(desc) = &milestone.description {
        println!("{}\n", desc);
    }

    let total = milestone.open_issues + milestone.closed_issues;
    let pct = milestone.completion_percent();

    println!("- **State:** {:?}", milestone.state);
    println!(
        "- **Progress:** {:.1}% ({}/{})",
        pct, milestone.closed_issues, total
    );
    println!("- **Open Issues:** {}", milestone.open_issues);
    println!("- **Closed Issues:** {}", milestone.closed_issues);

    if let Some(due) = milestone.due_on {
        println!("- **Due Date:** {}", due.format("%Y-%m-%d"));
        if milestone.is_overdue() {
            println!("\n> **Warning:** This milestone is overdue!");
        }
    }

    Ok(())
}
