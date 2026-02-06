//! Burndown chart report generation

use crate::models::{Issue, Milestone};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Burndown chart report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownReport {
    /// Milestone name
    pub milestone: String,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// Due date (if set)
    pub end_date: Option<DateTime<Utc>>,
    /// Total issues in milestone
    pub total_issues: usize,
    /// Actual burndown data points
    pub data_points: Vec<BurndownDataPoint>,
    /// Ideal burndown line
    pub ideal_burndown: Vec<BurndownDataPoint>,
    /// Projected completion date
    pub projected_completion: Option<DateTime<Utc>>,
}

/// Single burndown data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownDataPoint {
    /// Date
    pub date: DateTime<Utc>,
    /// Remaining issues
    pub remaining: usize,
    /// Completed issues
    pub completed: usize,
}

/// Calculator for burndown reports
pub struct BurndownCalculator;

impl BurndownCalculator {
    /// Generate a burndown report for a milestone
    pub fn calculate(issues: &[Issue], milestone: &Milestone) -> BurndownReport {
        let milestone_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(milestone.id))
            .collect();

        let total = milestone_issues.len();
        let start_date = milestone.created_at;
        let end_date = milestone.due_on;

        // Generate daily data points
        let mut data_points = Vec::new();
        let mut current = start_date;
        let now = Utc::now();

        while current <= now {
            let completed_by_date = milestone_issues
                .iter()
                .filter(|i| i.closed_at.map(|c| c <= current).unwrap_or(false))
                .count();

            data_points.push(BurndownDataPoint {
                date: current,
                remaining: total - completed_by_date,
                completed: completed_by_date,
            });

            current += Duration::days(1);
        }

        // Calculate ideal burndown
        let ideal_burndown = if let Some(end) = end_date {
            let days = (end - start_date).num_days().max(1) as usize;
            let daily_rate = total as f64 / days as f64;

            (0..=days)
                .map(|day| {
                    let date = start_date + Duration::days(day as i64);
                    let remaining = (total as f64 - (day as f64 * daily_rate)).max(0.0) as usize;
                    BurndownDataPoint {
                        date,
                        remaining,
                        completed: total - remaining,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Project completion date
        let projected_completion = Self::project_completion(&data_points, total);

        BurndownReport {
            milestone: milestone.title.clone(),
            start_date,
            end_date,
            total_issues: total,
            data_points,
            ideal_burndown,
            projected_completion,
        }
    }

    fn project_completion(
        data_points: &[BurndownDataPoint],
        _total: usize,
    ) -> Option<DateTime<Utc>> {
        if data_points.len() < 2 {
            return None;
        }

        // Use last 7 days to calculate velocity
        let recent: Vec<_> = data_points.iter().rev().take(7).collect();
        if recent.len() < 2 {
            return None;
        }

        let first = recent.last().unwrap();
        let last = recent.first().unwrap();

        let days = (last.date - first.date).num_days() as f64;
        let completed_diff = last.completed as f64 - first.completed as f64;

        if completed_diff <= 0.0 || days <= 0.0 {
            return None;
        }

        let daily_rate = completed_diff / days;
        let remaining = last.remaining as f64;

        if remaining <= 0.0 {
            return Some(last.date);
        }

        let days_to_complete = (remaining / daily_rate).ceil() as i64;
        Some(last.date + Duration::days(days_to_complete))
    }
}

/// Burnup chart report (inverse of burndown)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnupReport {
    /// Milestone name
    pub milestone: String,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// Due date
    pub end_date: Option<DateTime<Utc>>,
    /// Scope changes over time
    pub scope_data: Vec<ScopeDataPoint>,
    /// Work completed over time
    pub completed_data: Vec<BurndownDataPoint>,
}

/// Scope data point for burnup chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeDataPoint {
    /// Date
    pub date: DateTime<Utc>,
    /// Total scope (issues) at this point
    pub total_scope: usize,
}

impl BurndownCalculator {
    /// Generate a burnup report
    pub fn calculate_burnup(issues: &[Issue], milestone: &Milestone) -> BurnupReport {
        let milestone_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(milestone.id))
            .collect();

        let start_date = milestone.created_at;
        let end_date = milestone.due_on;
        let now = Utc::now();

        let mut scope_data = Vec::new();
        let mut completed_data = Vec::new();
        let mut current = start_date;

        while current <= now {
            // Count issues added by this date
            let scope = milestone_issues
                .iter()
                .filter(|i| i.created_at <= current)
                .count();

            scope_data.push(ScopeDataPoint {
                date: current,
                total_scope: scope,
            });

            // Count completed by this date
            let completed = milestone_issues
                .iter()
                .filter(|i| i.closed_at.map(|c| c <= current).unwrap_or(false))
                .count();

            completed_data.push(BurndownDataPoint {
                date: current,
                remaining: scope - completed,
                completed,
            });

            current += Duration::days(1);
        }

        BurnupReport {
            milestone: milestone.title.clone(),
            start_date,
            end_date,
            scope_data,
            completed_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IssueState, MilestoneState, User};

    fn create_test_user() -> User {
        User {
            id: 1,
            login: "test".to_string(),
            avatar_url: "".to_string(),
            html_url: "".to_string(),
        }
    }

    fn create_test_milestone(days_old: i64, due_in_days: Option<i64>) -> Milestone {
        let now = Utc::now();
        Milestone {
            id: 1,
            number: 1,
            title: "v1.0".to_string(),
            description: Some("Test milestone".to_string()),
            state: MilestoneState::Open,
            open_issues: 5,
            closed_issues: 3,
            due_on: due_in_days.map(|d| now + Duration::days(d)),
            created_at: now - Duration::days(days_old),
            closed_at: None,
        }
    }

    fn create_test_issue(
        number: u64,
        milestone_id: Option<i64>,
        days_old: i64,
        closed_days_ago: Option<i64>,
    ) -> Issue {
        let now = Utc::now();
        let milestone = milestone_id.map(|id| Milestone {
            id,
            number: 1,
            title: "v1.0".to_string(),
            description: None,
            state: MilestoneState::Open,
            open_issues: 0,
            closed_issues: 0,
            due_on: None,
            created_at: now - Duration::days(14),
            closed_at: None,
        });

        Issue {
            id: number as i64,
            number,
            title: format!("Issue #{}", number),
            body: None,
            state: if closed_days_ago.is_some() {
                IssueState::Closed
            } else {
                IssueState::Open
            },
            labels: vec![],
            assignees: vec![],
            milestone,
            author: create_test_user(),
            comments_count: 0,
            created_at: now - Duration::days(days_old),
            updated_at: now,
            closed_at: closed_days_ago.map(|d| now - Duration::days(d)),
            closed_by: None,
        }
    }

    #[test]
    fn test_burndown_basic() {
        let milestone = create_test_milestone(14, Some(14));
        let issues = vec![
            create_test_issue(1, Some(1), 14, Some(7)), // Closed 7 days ago
            create_test_issue(2, Some(1), 14, Some(3)), // Closed 3 days ago
            create_test_issue(3, Some(1), 14, None),    // Still open
            create_test_issue(4, Some(1), 14, None),    // Still open
        ];

        let report = BurndownCalculator::calculate(&issues, &milestone);

        assert_eq!(report.milestone, "v1.0");
        assert_eq!(report.total_issues, 4);
        assert!(!report.data_points.is_empty());
        // First data point should have all 4 remaining
        assert_eq!(report.data_points.first().unwrap().remaining, 4);
        // Last data point should have 2 remaining
        assert_eq!(report.data_points.last().unwrap().remaining, 2);
    }

    #[test]
    fn test_burndown_with_due_date_generates_ideal_line() {
        let milestone = create_test_milestone(14, Some(14));
        let issues = vec![
            create_test_issue(1, Some(1), 14, None),
            create_test_issue(2, Some(1), 14, None),
        ];

        let report = BurndownCalculator::calculate(&issues, &milestone);

        // Should have ideal burndown because there's a due date
        assert!(!report.ideal_burndown.is_empty());
        // Ideal should start at total and end at 0
        assert_eq!(report.ideal_burndown.first().unwrap().remaining, 2);
        assert_eq!(report.ideal_burndown.last().unwrap().remaining, 0);
    }

    #[test]
    fn test_burndown_without_due_date() {
        let milestone = create_test_milestone(7, None);
        let issues = vec![
            create_test_issue(1, Some(1), 7, None),
            create_test_issue(2, Some(1), 7, None),
        ];

        let report = BurndownCalculator::calculate(&issues, &milestone);

        // Without due date, no ideal line
        assert!(report.ideal_burndown.is_empty());
        assert!(report.end_date.is_none());
    }

    #[test]
    fn test_burndown_filters_by_milestone() {
        let milestone = create_test_milestone(7, Some(7));
        let issues = vec![
            create_test_issue(1, Some(1), 7, None), // In milestone
            create_test_issue(2, Some(1), 7, None), // In milestone
            create_test_issue(3, Some(2), 7, None), // Different milestone
            create_test_issue(4, None, 7, None),    // No milestone
        ];

        let report = BurndownCalculator::calculate(&issues, &milestone);
        assert_eq!(report.total_issues, 2);
    }

    #[test]
    fn test_burnup_report() {
        let milestone = create_test_milestone(14, Some(14));
        let issues = vec![
            create_test_issue(1, Some(1), 14, Some(7)),
            create_test_issue(2, Some(1), 10, Some(3)), // Added later
            create_test_issue(3, Some(1), 7, None),     // Added even later
        ];

        let report = BurndownCalculator::calculate_burnup(&issues, &milestone);

        assert_eq!(report.milestone, "v1.0");
        assert!(!report.scope_data.is_empty());
        assert!(!report.completed_data.is_empty());

        // Scope should increase over time as issues were added
        let first_scope = report.scope_data.first().unwrap().total_scope;
        let last_scope = report.scope_data.last().unwrap().total_scope;
        assert!(last_scope >= first_scope);
    }

    #[test]
    fn test_project_completion_calculation() {
        // Create data points showing steady progress
        let now = Utc::now();
        let data_points: Vec<BurndownDataPoint> = (0..10)
            .map(|day| BurndownDataPoint {
                date: now - Duration::days(9 - day),
                remaining: (10 - day) as usize,
                completed: day as usize,
            })
            .collect();

        let projected = BurndownCalculator::project_completion(&data_points, 10);
        assert!(projected.is_some());
        // Should project completion at or after now
        assert!(projected.unwrap() >= now - Duration::days(1));
    }

    #[test]
    fn test_project_completion_no_progress() {
        let now = Utc::now();
        // No progress at all
        let data_points: Vec<BurndownDataPoint> = (0..5)
            .map(|day| BurndownDataPoint {
                date: now - Duration::days(4 - day),
                remaining: 10,
                completed: 0,
            })
            .collect();

        let projected = BurndownCalculator::project_completion(&data_points, 10);
        // Can't project when no progress is made
        assert!(projected.is_none());
    }
}
