//! Table output formatting

use colored::Colorize;
use comfy_table::{Cell, Color, Table};
use greport_core::metrics::{IssueMetrics, PullMetrics, SlaReport, VelocityMetrics};
use greport_core::models::{Issue, IssueState, Milestone, PullRequest, PullState, Release};
use greport_core::reports::BurndownReport;

pub fn format_issues(issues: &[Issue]) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_header(vec!["#", "Title", "State", "Labels", "Assignee", "Age"]);

    for issue in issues {
        let state_cell = if issue.state == IssueState::Open {
            Cell::new("open").fg(Color::Green)
        } else {
            Cell::new("closed").fg(Color::Red)
        };

        let labels = issue
            .labels
            .iter()
            .take(3)
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let assignee = issue
            .assignees
            .first()
            .map(|a| a.login.as_str())
            .unwrap_or("-");

        let title = if issue.title.len() > 50 {
            format!("{}...", &issue.title[..47])
        } else {
            issue.title.clone()
        };

        table.add_row(vec![
            Cell::new(issue.number),
            Cell::new(title),
            state_cell,
            Cell::new(labels),
            Cell::new(assignee),
            Cell::new(format!("{}d", issue.age_days())),
        ]);
    }

    println!("{table}");
    println!("\nTotal: {} issues", issues.len());
    Ok(())
}

pub fn format_issue_metrics(metrics: &IssueMetrics) -> anyhow::Result<()> {
    println!("{}", "Issue Metrics".bold());
    println!("{}", "=".repeat(40));

    println!("Total:        {}", metrics.total);
    println!(
        "Open:         {} ({}%)",
        metrics.open.to_string().green(),
        if metrics.total > 0 {
            metrics.open * 100 / metrics.total
        } else {
            0
        }
    );
    println!(
        "Closed:       {} ({}%)",
        metrics.closed.to_string().red(),
        if metrics.total > 0 {
            metrics.closed * 100 / metrics.total
        } else {
            0
        }
    );
    println!("Stale:        {}", metrics.stale_count.to_string().yellow());

    if let Some(avg) = metrics.avg_time_to_close_hours {
        println!("\nAvg time to close: {:.1} hours ({:.1} days)", avg, avg / 24.0);
    }

    if let Some(median) = metrics.median_time_to_close_hours {
        println!("Median time to close: {:.1} hours ({:.1} days)", median, median / 24.0);
    }

    println!("\n{}", "By Label:".bold());
    let mut labels: Vec<_> = metrics.by_label.iter().collect();
    labels.sort_by(|a, b| b.1.cmp(a.1));
    for (label, count) in labels.iter().take(10) {
        println!("  {}: {}", label, count);
    }

    println!("\n{}", "Age Distribution:".bold());
    for bucket in &metrics.age_distribution.buckets {
        let bar = "#".repeat(bucket.count.min(30));
        println!("  {:>12}: {:>4} {}", bucket.label, bucket.count, bar);
    }

    Ok(())
}

pub fn format_velocity(velocity: &VelocityMetrics) -> anyhow::Result<()> {
    println!("{}", "Velocity Report".bold());
    println!("{}", "=".repeat(60));

    let mut table = Table::new();
    table.set_header(vec!["Period", "Opened", "Closed", "Net", "Open Total"]);

    for dp in &velocity.data_points {
        let net_cell = if dp.net_change > 0 {
            Cell::new(format!("+{}", dp.net_change)).fg(Color::Red)
        } else if dp.net_change < 0 {
            Cell::new(dp.net_change.to_string()).fg(Color::Green)
        } else {
            Cell::new("0")
        };

        table.add_row(vec![
            Cell::new(dp.period_start.format("%Y-%m-%d").to_string()),
            Cell::new(dp.opened),
            Cell::new(dp.closed),
            net_cell,
            Cell::new(dp.cumulative_open),
        ]);
    }

    println!("{table}");
    println!("\nAverage opened per {}: {:.1}", velocity.period.label(), velocity.avg_opened);
    println!("Average closed per {}: {:.1}", velocity.period.label(), velocity.avg_closed);
    println!("Trend: {}", velocity.trend.label());

    Ok(())
}

pub fn format_burndown(burndown: &BurndownReport) -> anyhow::Result<()> {
    println!("{}", format!("Burndown: {}", burndown.milestone).bold());
    println!("{}", "=".repeat(50));

    println!("Total issues: {}", burndown.total_issues);
    println!("Start date: {}", burndown.start_date.format("%Y-%m-%d"));

    if let Some(end) = burndown.end_date {
        println!("Due date: {}", end.format("%Y-%m-%d"));
    }

    if let Some(projected) = burndown.projected_completion {
        println!("Projected completion: {}", projected.format("%Y-%m-%d"));
    }

    println!("\n{}", "Progress:".bold());
    if let Some(last) = burndown.data_points.last() {
        let completed_pct = if burndown.total_issues > 0 {
            (last.completed * 100) / burndown.total_issues
        } else {
            0
        };

        let bar_width = 40;
        let filled = (completed_pct * bar_width) / 100;
        let empty = bar_width - filled;

        println!(
            "[{}{}] {}% ({}/{})",
            "#".repeat(filled).green(),
            "-".repeat(empty),
            completed_pct,
            last.completed,
            burndown.total_issues
        );
    }

    Ok(())
}

pub fn format_sla(sla: &SlaReport) -> anyhow::Result<()> {
    println!("{}", "SLA Compliance Report".bold());
    println!("{}", "=".repeat(50));

    println!("Total issues evaluated: {}", sla.total_issues);
    println!();

    println!("{}", "Response SLA:".bold());
    println!(
        "  Met: {} | Breached: {} | Compliance: {:.1}%",
        sla.response_sla_met.to_string().green(),
        sla.response_sla_breached.to_string().red(),
        sla.response_compliance_percent
    );

    println!("\n{}", "Resolution SLA:".bold());
    println!(
        "  Met: {} | Breached: {} | Compliance: {:.1}%",
        sla.resolution_sla_met.to_string().green(),
        sla.resolution_sla_breached.to_string().red(),
        sla.resolution_compliance_percent
    );

    if !sla.violations.is_empty() {
        println!("\n{}", "Violations:".bold().red());
        let mut table = Table::new();
        table.set_header(vec!["Issue", "Type", "SLA", "Actual", "Exceeded By"]);

        for v in sla.violations.iter().take(10) {
            table.add_row(vec![
                Cell::new(format!("#{}", v.issue_number)),
                Cell::new(format!("{:?}", v.violation_type)),
                Cell::new(format!("{}h", v.sla_hours)),
                Cell::new(format!("{}h", v.actual_hours)),
                Cell::new(format!("{}h", v.exceeded_by_hours)).fg(Color::Red),
            ]);
        }
        println!("{table}");

        if sla.violations.len() > 10 {
            println!("... and {} more violations", sla.violations.len() - 10);
        }
    }

    Ok(())
}

pub fn format_pulls(prs: &[PullRequest]) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_header(vec!["#", "Title", "State", "Author", "Size", "Age"]);

    for pr in prs {
        let state_cell = if pr.merged {
            Cell::new("merged").fg(Color::Magenta)
        } else if pr.state == PullState::Open {
            Cell::new("open").fg(Color::Green)
        } else {
            Cell::new("closed").fg(Color::Red)
        };

        let title = if pr.title.len() > 45 {
            format!("{}...", &pr.title[..42])
        } else {
            pr.title.clone()
        };

        let age = (chrono::Utc::now() - pr.created_at).num_days();

        table.add_row(vec![
            Cell::new(pr.number),
            Cell::new(title),
            state_cell,
            Cell::new(&pr.author.login),
            Cell::new(pr.size_category().label()),
            Cell::new(format!("{}d", age)),
        ]);
    }

    println!("{table}");
    println!("\nTotal: {} pull requests", prs.len());
    Ok(())
}

pub fn format_pull_metrics(metrics: &PullMetrics) -> anyhow::Result<()> {
    println!("{}", "Pull Request Metrics".bold());
    println!("{}", "=".repeat(40));

    println!("Total:           {}", metrics.total);
    println!("Open:            {}", metrics.open.to_string().green());
    println!("Merged:          {}", metrics.merged.to_string().magenta());
    println!("Closed (unmerged): {}", metrics.closed_unmerged);
    println!("Drafts:          {}", metrics.draft_count);

    if let Some(avg) = metrics.avg_time_to_merge_hours {
        println!("\nAvg time to merge: {:.1} hours ({:.1} days)", avg, avg / 24.0);
    }

    println!("\n{}", "By Size:".bold());
    for (size, count) in &metrics.by_size {
        println!("  {}: {}", size, count);
    }

    Ok(())
}

pub fn format_releases(releases: &[Release]) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_header(vec!["Tag", "Name", "Type", "Date", "Author"]);

    for release in releases {
        let type_cell = if release.draft {
            Cell::new("draft").fg(Color::Yellow)
        } else if release.prerelease {
            Cell::new("prerelease").fg(Color::Cyan)
        } else {
            Cell::new("release").fg(Color::Green)
        };

        let name = release.name.as_deref().unwrap_or("-");
        let date = release
            .published_at
            .unwrap_or(release.created_at)
            .format("%Y-%m-%d")
            .to_string();

        table.add_row(vec![
            Cell::new(&release.tag_name),
            Cell::new(name),
            type_cell,
            Cell::new(date),
            Cell::new(&release.author.login),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn format_milestone_progress(milestone: &Milestone) -> anyhow::Result<()> {
    println!("{}", format!("Milestone: {}", milestone.title).bold());
    println!("{}", "=".repeat(50));

    let total = milestone.open_issues + milestone.closed_issues;
    let completion = milestone.completion_percent();

    println!("State: {:?}", milestone.state);
    println!("Open issues: {}", milestone.open_issues);
    println!("Closed issues: {}", milestone.closed_issues);

    if let Some(due) = milestone.due_on {
        println!("Due date: {}", due.format("%Y-%m-%d"));
        if milestone.is_overdue() {
            println!("{}", "STATUS: OVERDUE".red().bold());
        }
    }

    println!("\n{}", "Progress:".bold());
    let bar_width = 40;
    let filled = (completion as usize * bar_width) / 100;
    let empty = bar_width - filled;

    println!(
        "[{}{}] {:.1}% ({}/{})",
        "#".repeat(filled).green(),
        "-".repeat(empty),
        completion,
        milestone.closed_issues,
        total
    );

    Ok(())
}
