//! Issue model and related types

use super::User;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Issue state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    /// Issue is open
    Open,
    /// Issue is closed
    Closed,
}

/// Issue label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Label ID
    pub id: i64,
    /// Label name
    pub name: String,
    /// Label color (hex)
    pub color: String,
    /// Label description
    pub description: Option<String>,
}

/// Milestone state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneState {
    /// Milestone is open
    Open,
    /// Milestone is closed
    Closed,
}

/// Milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone ID
    pub id: i64,
    /// Milestone number
    pub number: u64,
    /// Milestone title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Milestone state
    pub state: MilestoneState,
    /// Number of open issues
    pub open_issues: u32,
    /// Number of closed issues
    pub closed_issues: u32,
    /// Due date
    pub due_on: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Closed timestamp
    pub closed_at: Option<DateTime<Utc>>,
}

impl Milestone {
    /// Calculate completion percentage
    pub fn completion_percent(&self) -> f64 {
        let total = self.open_issues + self.closed_issues;
        if total == 0 {
            return 0.0;
        }
        (self.closed_issues as f64 / total as f64) * 100.0
    }

    /// Check if milestone is overdue
    pub fn is_overdue(&self) -> bool {
        match (self.due_on, self.state) {
            (Some(due), MilestoneState::Open) => due < Utc::now(),
            _ => false,
        }
    }
}

/// GitHub issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID
    pub id: i64,
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Issue body (markdown)
    pub body: Option<String>,
    /// Issue state
    pub state: IssueState,
    /// Labels attached to the issue
    pub labels: Vec<Label>,
    /// Assigned users
    pub assignees: Vec<User>,
    /// Associated milestone
    pub milestone: Option<Milestone>,
    /// Issue author
    pub author: User,
    /// Number of comments
    pub comments_count: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Closed timestamp
    pub closed_at: Option<DateTime<Utc>>,
    /// User who closed the issue
    pub closed_by: Option<User>,
}

impl Issue {
    /// Calculate the age of the issue in days
    pub fn age_days(&self) -> i64 {
        let end = self.closed_at.unwrap_or_else(Utc::now);
        (end - self.created_at).num_days()
    }

    /// Calculate time to close in hours (None if still open)
    pub fn time_to_close_hours(&self) -> Option<i64> {
        self.closed_at
            .map(|closed| (closed - self.created_at).num_hours())
    }

    /// Calculate time to close in days (None if still open)
    pub fn time_to_close_days(&self) -> Option<i64> {
        self.closed_at
            .map(|closed| (closed - self.created_at).num_days())
    }

    /// Check if issue is stale (no update in N days)
    pub fn is_stale(&self, days: i64) -> bool {
        let threshold = Utc::now() - Duration::days(days);
        self.updated_at < threshold && self.state == IssueState::Open
    }

    /// Check if issue has a specific label (case-insensitive)
    pub fn has_label(&self, name: &str) -> bool {
        self.labels
            .iter()
            .any(|l| l.name.eq_ignore_ascii_case(name))
    }

    /// Get label names
    pub fn label_names(&self) -> Vec<&str> {
        self.labels.iter().map(|l| l.name.as_str()).collect()
    }

    /// Get assignee logins
    pub fn assignee_logins(&self) -> Vec<&str> {
        self.assignees.iter().map(|a| a.login.as_str()).collect()
    }

    /// Check if issue is assigned
    pub fn is_assigned(&self) -> bool {
        !self.assignees.is_empty()
    }
}

/// Issue timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEvent {
    /// Event ID
    pub id: i64,
    /// Event type (labeled, assigned, closed, etc.)
    pub event_type: String,
    /// Actor who triggered the event
    pub actor: Option<User>,
    /// Event timestamp
    pub created_at: DateTime<Utc>,
    /// Label name (for label events)
    pub label_name: Option<String>,
    /// Assignee (for assignment events)
    pub assignee: Option<User>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_issue(state: IssueState, created_days_ago: i64) -> Issue {
        Issue {
            id: 1,
            number: 1,
            title: "Test issue".to_string(),
            body: None,
            state,
            labels: vec![],
            assignees: vec![],
            milestone: None,
            author: User::unknown(),
            comments_count: 0,
            created_at: Utc::now() - Duration::days(created_days_ago),
            updated_at: Utc::now() - Duration::days(created_days_ago),
            closed_at: None,
            closed_by: None,
        }
    }

    #[test]
    fn test_issue_age() {
        let issue = create_test_issue(IssueState::Open, 5);
        assert_eq!(issue.age_days(), 5);
    }

    #[test]
    fn test_issue_is_stale() {
        let mut issue = create_test_issue(IssueState::Open, 35);
        issue.updated_at = Utc::now() - Duration::days(35);
        assert!(issue.is_stale(30));

        let fresh_issue = create_test_issue(IssueState::Open, 5);
        assert!(!fresh_issue.is_stale(30));
    }

    #[test]
    fn test_issue_has_label() {
        let mut issue = create_test_issue(IssueState::Open, 1);
        issue.labels.push(Label {
            id: 1,
            name: "bug".to_string(),
            color: "ff0000".to_string(),
            description: None,
        });

        assert!(issue.has_label("bug"));
        assert!(issue.has_label("BUG")); // Case insensitive
        assert!(!issue.has_label("feature"));
    }
}
