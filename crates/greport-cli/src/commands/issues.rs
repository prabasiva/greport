//! Issue command handlers

use crate::args::{IssuesCommands, OutputFormat};
use crate::output::Formatter;
use greport_core::client::{GitHubClient, IssueParams, RepoId};
use greport_core::metrics::{IssueMetricsCalculator, SlaCalculator, VelocityCalculator};
use greport_core::reports::BurndownCalculator;
use greport_core::Config;
use std::collections::HashMap;

pub async fn handle_issues(
    client: &impl GitHubClient,
    repo: &RepoId,
    command: IssuesCommands,
    format: OutputFormat,
    config: &Config,
) -> anyhow::Result<()> {
    let formatter = Formatter::new(format);

    match command {
        IssuesCommands::List {
            state,
            labels,
            assignee,
            milestone,
            since,
            limit,
        } => {
            let mut params = IssueParams {
                state: state.into(),
                labels: labels.map(|l| l.split(',').map(String::from).collect()),
                assignee,
                milestone,
                per_page: limit.min(100),
                ..Default::default()
            };

            if let Some(since_str) = since {
                let date = chrono::NaiveDate::parse_from_str(&since_str, "%Y-%m-%d")?;
                let datetime = date.and_hms_opt(0, 0, 0).unwrap();
                params.since = Some(chrono::DateTime::from_naive_utc_and_offset(
                    datetime,
                    chrono::Utc,
                ));
            }

            let issues = client.list_issues(repo, params).await?;
            let issues: Vec<_> = issues.into_iter().take(limit).collect();
            formatter.format_issues(&issues)?;
        }

        IssuesCommands::Count { group_by, state } => {
            let params = IssueParams {
                state: state.map(Into::into).unwrap_or(greport_core::client::IssueStateFilter::All),
                ..Default::default()
            };

            let issues = client.list_issues(repo, params).await?;
            let calculator = IssueMetricsCalculator::new(30);
            let metrics = calculator.calculate(&issues);

            // Print based on group_by
            match group_by {
                crate::args::GroupBy::State => {
                    println!("Open: {}", metrics.open);
                    println!("Closed: {}", metrics.closed);
                }
                crate::args::GroupBy::Label => {
                    let mut labels: Vec<_> = metrics.by_label.iter().collect();
                    labels.sort_by(|a, b| b.1.cmp(a.1));
                    for (label, count) in labels {
                        println!("{}: {}", label, count);
                    }
                }
                crate::args::GroupBy::Assignee => {
                    let mut assignees: Vec<_> = metrics.by_assignee.iter().collect();
                    assignees.sort_by(|a, b| b.1.cmp(a.1));
                    for (assignee, count) in assignees {
                        println!("{}: {}", assignee, count);
                    }
                }
                crate::args::GroupBy::Milestone => {
                    let mut milestones: Vec<_> = metrics.by_milestone.iter().collect();
                    milestones.sort_by(|a, b| b.1.cmp(a.1));
                    for (milestone, count) in milestones {
                        println!("{}: {}", milestone, count);
                    }
                }
            }
        }

        IssuesCommands::Age { open_only } => {
            let params = if open_only {
                IssueParams::open()
            } else {
                IssueParams::all()
            };

            let issues = client.list_issues(repo, params).await?;
            let calculator = IssueMetricsCalculator::new(30);
            let metrics = calculator.calculate(&issues);

            println!("Age Distribution:");
            for bucket in &metrics.age_distribution.buckets {
                let bar = "#".repeat(bucket.count.min(50));
                println!("{:>12}: {:>4} {}", bucket.label, bucket.count, bar);
            }
        }

        IssuesCommands::Stale { days } => {
            let issues = client.list_issues(repo, IssueParams::open()).await?;
            let stale: Vec<_> = issues.into_iter().filter(|i| i.is_stale(days)).collect();

            println!("Found {} stale issues (no activity in {} days):\n", stale.len(), days);
            formatter.format_issues(&stale)?;
        }

        IssuesCommands::Velocity { period, last } => {
            let issues = client.list_issues(repo, IssueParams::all()).await?;
            let velocity = VelocityCalculator::calculate(&issues, period.into(), last);
            formatter.format_velocity(&velocity)?;
        }

        IssuesCommands::Burndown { milestone } => {
            let milestones = client.list_milestones(repo).await?;
            let ms = milestones
                .iter()
                .find(|m| m.title.eq_ignore_ascii_case(&milestone))
                .ok_or_else(|| anyhow::anyhow!("Milestone not found: {}", milestone))?;

            let issues = client.list_issues(repo, IssueParams::all()).await?;
            let burndown = BurndownCalculator::calculate(&issues, ms);
            formatter.format_burndown(&burndown)?;
        }

        IssuesCommands::Sla => {
            let issues = client.list_issues(repo, IssueParams::all()).await?;

            // Get events for each issue (simplified - would need batching for large repos)
            let mut events_map: HashMap<u64, Vec<greport_core::models::IssueEvent>> = HashMap::new();
            for issue in issues.iter().take(50) {
                if let Ok(events) = client.list_issue_events(repo, issue.number).await {
                    events_map.insert(issue.number, events);
                }
            }

            let calculator = SlaCalculator::new(config.sla.clone());
            let report = calculator.calculate(&issues, &events_map);
            formatter.format_sla(&report)?;
        }

        IssuesCommands::Metrics => {
            let issues = client.list_issues(repo, IssueParams::all()).await?;
            let calculator = IssueMetricsCalculator::new(30);
            let metrics = calculator.calculate(&issues);
            formatter.format_issue_metrics(&metrics)?;
        }
    }

    Ok(())
}
