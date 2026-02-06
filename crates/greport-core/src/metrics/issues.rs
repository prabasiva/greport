//! Issue metrics calculations

use crate::models::{Issue, IssueState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated issue metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMetrics {
    /// Total number of issues
    pub total: usize,
    /// Number of open issues
    pub open: usize,
    /// Number of closed issues
    pub closed: usize,
    /// Average time to close (hours)
    pub avg_time_to_close_hours: Option<f64>,
    /// Median time to close (hours)
    pub median_time_to_close_hours: Option<f64>,
    /// Issues grouped by label
    pub by_label: HashMap<String, usize>,
    /// Issues grouped by assignee
    pub by_assignee: HashMap<String, usize>,
    /// Issues grouped by milestone
    pub by_milestone: HashMap<String, usize>,
    /// Age distribution buckets
    pub age_distribution: AgeDistribution,
    /// Number of stale issues
    pub stale_count: usize,
}

/// Age distribution for issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeDistribution {
    /// Age buckets
    pub buckets: Vec<AgeBucket>,
}

/// Single age bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeBucket {
    /// Display label
    pub label: String,
    /// Minimum days (inclusive)
    pub min_days: i64,
    /// Maximum days (exclusive, None for unbounded)
    pub max_days: Option<i64>,
    /// Number of issues in this bucket
    pub count: usize,
}

/// Calculator for issue metrics
pub struct IssueMetricsCalculator {
    stale_days: i64,
}

impl IssueMetricsCalculator {
    /// Create a new calculator with the given stale threshold
    pub fn new(stale_days: i64) -> Self {
        Self { stale_days }
    }

    /// Calculate metrics from a list of issues
    pub fn calculate(&self, issues: &[Issue]) -> IssueMetrics {
        let open: Vec<_> = issues
            .iter()
            .filter(|i| i.state == IssueState::Open)
            .collect();
        let closed: Vec<_> = issues
            .iter()
            .filter(|i| i.state == IssueState::Closed)
            .collect();

        // Calculate time to close
        let close_times: Vec<i64> = closed
            .iter()
            .filter_map(|i| i.time_to_close_hours())
            .collect();

        let avg_time = if close_times.is_empty() {
            None
        } else {
            Some(close_times.iter().sum::<i64>() as f64 / close_times.len() as f64)
        };

        let median_time = Self::calculate_median(&close_times);

        // Group by label
        let mut by_label: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            for label in &issue.labels {
                *by_label.entry(label.name.clone()).or_insert(0) += 1;
            }
        }

        // Group by assignee
        let mut by_assignee: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            if issue.assignees.is_empty() {
                *by_assignee.entry("Unassigned".to_string()).or_insert(0) += 1;
            } else {
                for assignee in &issue.assignees {
                    *by_assignee.entry(assignee.login.clone()).or_insert(0) += 1;
                }
            }
        }

        // Group by milestone
        let mut by_milestone: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            let milestone_name = issue
                .milestone
                .as_ref()
                .map(|m| m.title.clone())
                .unwrap_or_else(|| "No Milestone".to_string());
            *by_milestone.entry(milestone_name).or_insert(0) += 1;
        }

        // Calculate stale issues
        let stale_count = open.iter().filter(|i| i.is_stale(self.stale_days)).count();

        // Age distribution
        let age_distribution = self.calculate_age_distribution(&open);

        IssueMetrics {
            total: issues.len(),
            open: open.len(),
            closed: closed.len(),
            avg_time_to_close_hours: avg_time,
            median_time_to_close_hours: median_time,
            by_label,
            by_assignee,
            by_milestone,
            age_distribution,
            stale_count,
        }
    }

    fn calculate_median(values: &[i64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            Some((sorted[mid - 1] + sorted[mid]) as f64 / 2.0)
        } else {
            Some(sorted[mid] as f64)
        }
    }

    fn calculate_age_distribution(&self, open_issues: &[&Issue]) -> AgeDistribution {
        let mut buckets = vec![
            AgeBucket {
                label: "< 1 day".into(),
                min_days: 0,
                max_days: Some(1),
                count: 0,
            },
            AgeBucket {
                label: "1-7 days".into(),
                min_days: 1,
                max_days: Some(7),
                count: 0,
            },
            AgeBucket {
                label: "1-4 weeks".into(),
                min_days: 7,
                max_days: Some(28),
                count: 0,
            },
            AgeBucket {
                label: "1-3 months".into(),
                min_days: 28,
                max_days: Some(90),
                count: 0,
            },
            AgeBucket {
                label: "3-6 months".into(),
                min_days: 90,
                max_days: Some(180),
                count: 0,
            },
            AgeBucket {
                label: "> 6 months".into(),
                min_days: 180,
                max_days: None,
                count: 0,
            },
        ];

        for issue in open_issues {
            let age = issue.age_days();
            for bucket in &mut buckets {
                let in_bucket =
                    age >= bucket.min_days && bucket.max_days.is_none_or(|max| age < max);
                if in_bucket {
                    bucket.count += 1;
                    break;
                }
            }
        }

        AgeDistribution { buckets }
    }
}

impl Default for IssueMetricsCalculator {
    fn default() -> Self {
        Self::new(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;
    use chrono::{Duration, Utc};

    fn create_issue(state: IssueState, age_days: i64) -> Issue {
        let created = Utc::now() - Duration::days(age_days);
        Issue {
            id: 1,
            number: 1,
            title: "Test".to_string(),
            body: None,
            state,
            labels: vec![],
            assignees: vec![],
            milestone: None,
            author: User::unknown(),
            comments_count: 0,
            created_at: created,
            updated_at: created,
            closed_at: if state == IssueState::Closed {
                Some(Utc::now())
            } else {
                None
            },
            closed_by: None,
        }
    }

    #[test]
    fn test_issue_metrics() {
        let issues = vec![
            create_issue(IssueState::Open, 5),
            create_issue(IssueState::Open, 15),
            create_issue(IssueState::Closed, 10),
        ];

        let calculator = IssueMetricsCalculator::new(30);
        let metrics = calculator.calculate(&issues);

        assert_eq!(metrics.total, 3);
        assert_eq!(metrics.open, 2);
        assert_eq!(metrics.closed, 1);
    }

    #[test]
    fn test_median_calculation() {
        assert_eq!(
            IssueMetricsCalculator::calculate_median(&[1, 2, 3]),
            Some(2.0)
        );
        assert_eq!(
            IssueMetricsCalculator::calculate_median(&[1, 2, 3, 4]),
            Some(2.5)
        );
        assert_eq!(IssueMetricsCalculator::calculate_median(&[]), None);
    }
}
