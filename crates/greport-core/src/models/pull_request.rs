//! Pull request model and related types

use super::{Label, Milestone, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Pull request state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullState {
    /// PR is open
    Open,
    /// PR is closed (may or may not be merged)
    Closed,
}

/// Pull request size category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrSize {
    /// 0-10 lines
    XSmall,
    /// 11-50 lines
    Small,
    /// 51-200 lines
    Medium,
    /// 201-500 lines
    Large,
    /// 500+ lines
    XLarge,
}

impl PrSize {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            PrSize::XSmall => "XS",
            PrSize::Small => "S",
            PrSize::Medium => "M",
            PrSize::Large => "L",
            PrSize::XLarge => "XL",
        }
    }
}

/// GitHub pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR ID
    pub id: i64,
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// PR body (markdown)
    pub body: Option<String>,
    /// PR state
    pub state: PullState,
    /// Is draft PR
    pub draft: bool,
    /// PR author
    pub author: User,
    /// Labels
    pub labels: Vec<Label>,
    /// Associated milestone
    pub milestone: Option<Milestone>,
    /// Head branch ref
    pub head_ref: String,
    /// Base branch ref
    pub base_ref: String,
    /// Is merged
    pub merged: bool,
    /// Merged timestamp
    pub merged_at: Option<DateTime<Utc>>,
    /// Lines added
    pub additions: u32,
    /// Lines deleted
    pub deletions: u32,
    /// Files changed
    pub changed_files: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Closed timestamp
    pub closed_at: Option<DateTime<Utc>>,
}

impl PullRequest {
    /// Calculate total lines changed
    pub fn lines_changed(&self) -> u32 {
        self.additions + self.deletions
    }

    /// Get PR size category
    pub fn size_category(&self) -> PrSize {
        match self.lines_changed() {
            0..=10 => PrSize::XSmall,
            11..=50 => PrSize::Small,
            51..=200 => PrSize::Medium,
            201..=500 => PrSize::Large,
            _ => PrSize::XLarge,
        }
    }

    /// Calculate time to merge in hours (None if not merged)
    pub fn time_to_merge_hours(&self) -> Option<i64> {
        self.merged_at
            .map(|merged| (merged - self.created_at).num_hours())
    }

    /// Calculate time to merge in days (None if not merged)
    pub fn time_to_merge_days(&self) -> Option<f64> {
        self.merged_at
            .map(|merged| (merged - self.created_at).num_hours() as f64 / 24.0)
    }

    /// Check if PR is ready for review (not draft, not merged)
    pub fn is_ready_for_review(&self) -> bool {
        !self.draft && self.state == PullState::Open
    }
}

/// Pull request review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Review ID
    pub id: i64,
    /// Reviewer
    pub user: Option<User>,
    /// Review body
    pub body: Option<String>,
    /// Review state (approved, changes_requested, commented)
    pub state: String,
    /// Submitted timestamp
    pub submitted_at: Option<DateTime<Utc>>,
}

impl Review {
    /// Check if review is approved
    pub fn is_approved(&self) -> bool {
        self.state.eq_ignore_ascii_case("approved")
    }

    /// Check if changes were requested
    pub fn changes_requested(&self) -> bool {
        self.state.eq_ignore_ascii_case("changes_requested")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pr() -> PullRequest {
        PullRequest {
            id: 1,
            number: 1,
            title: "Test PR".to_string(),
            body: None,
            state: PullState::Open,
            draft: false,
            author: User::unknown(),
            labels: vec![],
            milestone: None,
            head_ref: "feature".to_string(),
            base_ref: "main".to_string(),
            merged: false,
            merged_at: None,
            additions: 100,
            deletions: 50,
            changed_files: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        }
    }

    #[test]
    fn test_lines_changed() {
        let pr = create_test_pr();
        assert_eq!(pr.lines_changed(), 150);
    }

    #[test]
    fn test_size_category() {
        let mut pr = create_test_pr();

        pr.additions = 5;
        pr.deletions = 3;
        assert_eq!(pr.size_category(), PrSize::XSmall);

        pr.additions = 30;
        pr.deletions = 10;
        assert_eq!(pr.size_category(), PrSize::Small);

        pr.additions = 100;
        pr.deletions = 50;
        assert_eq!(pr.size_category(), PrSize::Medium);

        pr.additions = 300;
        pr.deletions = 100;
        assert_eq!(pr.size_category(), PrSize::Large);

        pr.additions = 1000;
        pr.deletions = 500;
        assert_eq!(pr.size_category(), PrSize::XLarge);
    }
}
