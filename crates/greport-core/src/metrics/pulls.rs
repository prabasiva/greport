//! Pull request metrics calculations

use crate::models::{PullRequest, PullState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated pull request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullMetrics {
    /// Total number of PRs
    pub total: usize,
    /// Number of open PRs
    pub open: usize,
    /// Number of merged PRs
    pub merged: usize,
    /// Number of closed (not merged) PRs
    pub closed_unmerged: usize,
    /// Average time to merge (hours)
    pub avg_time_to_merge_hours: Option<f64>,
    /// Median time to merge (hours)
    pub median_time_to_merge_hours: Option<f64>,
    /// PRs by size category
    pub by_size: HashMap<String, usize>,
    /// PRs by author
    pub by_author: HashMap<String, usize>,
    /// PRs by base branch
    pub by_base_branch: HashMap<String, usize>,
    /// Number of draft PRs
    pub draft_count: usize,
}

/// Calculator for pull request metrics
pub struct PullMetricsCalculator;

impl PullMetricsCalculator {
    /// Calculate metrics from a list of pull requests
    pub fn calculate(prs: &[PullRequest]) -> PullMetrics {
        let open: Vec<_> = prs.iter().filter(|p| p.state == PullState::Open).collect();
        let merged: Vec<_> = prs.iter().filter(|p| p.merged).collect();
        let closed_unmerged: Vec<_> = prs
            .iter()
            .filter(|p| p.state == PullState::Closed && !p.merged)
            .collect();

        // Calculate merge times
        let merge_times: Vec<i64> = merged
            .iter()
            .filter_map(|p| p.time_to_merge_hours())
            .collect();

        let avg_time = if merge_times.is_empty() {
            None
        } else {
            Some(merge_times.iter().sum::<i64>() as f64 / merge_times.len() as f64)
        };

        let median_time = Self::calculate_median(&merge_times);

        // Group by size
        let mut by_size: HashMap<String, usize> = HashMap::new();
        for pr in prs {
            let size = pr.size_category();
            *by_size.entry(size.label().to_string()).or_insert(0) += 1;
        }

        // Group by author
        let mut by_author: HashMap<String, usize> = HashMap::new();
        for pr in prs {
            *by_author.entry(pr.author.login.clone()).or_insert(0) += 1;
        }

        // Group by base branch
        let mut by_base_branch: HashMap<String, usize> = HashMap::new();
        for pr in prs {
            *by_base_branch.entry(pr.base_ref.clone()).or_insert(0) += 1;
        }

        // Count drafts
        let draft_count = prs.iter().filter(|p| p.draft).count();

        PullMetrics {
            total: prs.len(),
            open: open.len(),
            merged: merged.len(),
            closed_unmerged: closed_unmerged.len(),
            avg_time_to_merge_hours: avg_time,
            median_time_to_merge_hours: median_time,
            by_size,
            by_author,
            by_base_branch,
            draft_count,
        }
    }

    fn calculate_median(values: &[i64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) as f64 / 2.0)
        } else {
            Some(sorted[mid] as f64)
        }
    }
}

/// List of PRs without reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreviewedPrs {
    /// PRs without any reviews
    pub prs: Vec<UnreviewedPrSummary>,
    /// Total count
    pub count: usize,
}

/// Summary of an unreviewed PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreviewedPrSummary {
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// PR author
    pub author: String,
    /// Age in days
    pub age_days: i64,
    /// Is draft
    pub draft: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;
    use chrono::{Duration, Utc};

    fn create_test_user(login: &str) -> User {
        User {
            id: 1,
            login: login.to_string(),
            avatar_url: "".to_string(),
            html_url: "".to_string(),
        }
    }

    fn create_test_pr(
        number: u64,
        state: PullState,
        merged: bool,
        merge_hours_ago: Option<i64>,
        additions: u32,
        deletions: u32,
        author: &str,
        draft: bool,
    ) -> PullRequest {
        let now = Utc::now();
        let created_hours_ago = merge_hours_ago.unwrap_or(24) + 48;
        PullRequest {
            id: number as i64,
            number,
            title: format!("PR #{}", number),
            body: None,
            state,
            draft,
            author: create_test_user(author),
            labels: vec![],
            milestone: None,
            head_ref: "feature".to_string(),
            base_ref: "main".to_string(),
            merged,
            merged_at: merge_hours_ago.map(|h| now - Duration::hours(h)),
            additions,
            deletions,
            changed_files: 1,
            created_at: now - Duration::hours(created_hours_ago),
            updated_at: now,
            closed_at: if state == PullState::Closed { Some(now) } else { None },
        }
    }

    #[test]
    fn test_pull_metrics_basic_counts() {
        let prs = vec![
            create_test_pr(1, PullState::Open, false, None, 10, 5, "alice", false),
            create_test_pr(2, PullState::Closed, true, Some(24), 20, 10, "bob", false),
            create_test_pr(3, PullState::Closed, false, None, 30, 15, "alice", false),
        ];

        let metrics = PullMetricsCalculator::calculate(&prs);
        assert_eq!(metrics.total, 3);
        assert_eq!(metrics.open, 1);
        assert_eq!(metrics.merged, 1);
        assert_eq!(metrics.closed_unmerged, 1);
    }

    #[test]
    fn test_pull_metrics_by_author() {
        let prs = vec![
            create_test_pr(1, PullState::Open, false, None, 10, 5, "alice", false),
            create_test_pr(2, PullState::Open, false, None, 20, 10, "bob", false),
            create_test_pr(3, PullState::Closed, true, Some(24), 30, 15, "alice", false),
        ];

        let metrics = PullMetricsCalculator::calculate(&prs);
        assert_eq!(*metrics.by_author.get("alice").unwrap(), 2);
        assert_eq!(*metrics.by_author.get("bob").unwrap(), 1);
    }

    #[test]
    fn test_pull_metrics_by_size() {
        let prs = vec![
            // XS: < 10 changes
            create_test_pr(1, PullState::Open, false, None, 3, 2, "alice", false),
            // S: 10-50 changes
            create_test_pr(2, PullState::Open, false, None, 20, 10, "bob", false),
            // M: 50-200 changes
            create_test_pr(3, PullState::Open, false, None, 100, 50, "charlie", false),
            // L: 200-500 changes
            create_test_pr(4, PullState::Open, false, None, 200, 100, "dave", false),
            // XL: > 500 changes
            create_test_pr(5, PullState::Open, false, None, 400, 200, "eve", false),
        ];

        let metrics = PullMetricsCalculator::calculate(&prs);
        assert_eq!(*metrics.by_size.get("XS").unwrap_or(&0), 1);
        assert_eq!(*metrics.by_size.get("S").unwrap_or(&0), 1);
        assert_eq!(*metrics.by_size.get("M").unwrap_or(&0), 1);
        assert_eq!(*metrics.by_size.get("L").unwrap_or(&0), 1);
        assert_eq!(*metrics.by_size.get("XL").unwrap_or(&0), 1);
    }

    #[test]
    fn test_pull_metrics_merge_time() {
        let prs = vec![
            // Merged 24 hours after creation (48-24=24h to merge)
            create_test_pr(1, PullState::Closed, true, Some(24), 10, 5, "alice", false),
            // Merged 48 hours after creation
            create_test_pr(2, PullState::Closed, true, Some(0), 20, 10, "bob", false),
        ];

        let metrics = PullMetricsCalculator::calculate(&prs);
        assert!(metrics.avg_time_to_merge_hours.is_some());
        assert!(metrics.median_time_to_merge_hours.is_some());
    }

    #[test]
    fn test_pull_metrics_draft_count() {
        let prs = vec![
            create_test_pr(1, PullState::Open, false, None, 10, 5, "alice", true),
            create_test_pr(2, PullState::Open, false, None, 20, 10, "bob", true),
            create_test_pr(3, PullState::Open, false, None, 30, 15, "charlie", false),
        ];

        let metrics = PullMetricsCalculator::calculate(&prs);
        assert_eq!(metrics.draft_count, 2);
    }

    #[test]
    fn test_pull_metrics_empty() {
        let prs: Vec<PullRequest> = vec![];
        let metrics = PullMetricsCalculator::calculate(&prs);

        assert_eq!(metrics.total, 0);
        assert_eq!(metrics.open, 0);
        assert_eq!(metrics.merged, 0);
        assert!(metrics.avg_time_to_merge_hours.is_none());
        assert!(metrics.median_time_to_merge_hours.is_none());
    }

    #[test]
    fn test_pull_metrics_median_calculation() {
        // Test odd number of values
        let values = vec![10, 20, 30, 40, 50];
        let median = PullMetricsCalculator::calculate_median(&values);
        assert_eq!(median, Some(30.0));

        // Test even number of values
        let values = vec![10, 20, 30, 40];
        let median = PullMetricsCalculator::calculate_median(&values);
        assert_eq!(median, Some(25.0));

        // Test empty
        let values: Vec<i64> = vec![];
        let median = PullMetricsCalculator::calculate_median(&values);
        assert!(median.is_none());
    }
}
