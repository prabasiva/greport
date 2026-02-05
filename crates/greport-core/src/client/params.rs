//! Query parameters for GitHub API requests

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Issue state filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStateFilter {
    /// Open issues only
    #[default]
    Open,
    /// Closed issues only
    Closed,
    /// All issues
    All,
}

impl std::str::FromStr for IssueStateFilter {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(IssueStateFilter::Open),
            "closed" => Ok(IssueStateFilter::Closed),
            "all" => Ok(IssueStateFilter::All),
            _ => Err(crate::Error::Custom(format!("Invalid issue state: {}", s))),
        }
    }
}

/// Pull request state filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullStateFilter {
    /// Open PRs only
    #[default]
    Open,
    /// Closed PRs only
    Closed,
    /// All PRs
    All,
}

impl std::str::FromStr for PullStateFilter {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(PullStateFilter::Open),
            "closed" => Ok(PullStateFilter::Closed),
            "all" => Ok(PullStateFilter::All),
            _ => Err(crate::Error::Custom(format!("Invalid PR state: {}", s))),
        }
    }
}

/// Issue sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSort {
    /// Sort by creation date
    #[default]
    Created,
    /// Sort by update date
    Updated,
    /// Sort by comment count
    Comments,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending order
    Asc,
    /// Descending order
    #[default]
    Desc,
}

/// Parameters for listing issues
#[derive(Debug, Clone, Default)]
pub struct IssueParams {
    /// Filter by state
    pub state: IssueStateFilter,
    /// Filter by labels
    pub labels: Option<Vec<String>>,
    /// Filter by assignee
    pub assignee: Option<String>,
    /// Filter by milestone
    pub milestone: Option<String>,
    /// Filter by creator
    pub creator: Option<String>,
    /// Filter by mentioned user
    pub mentioned: Option<String>,
    /// Filter by issues created after this date
    pub since: Option<DateTime<Utc>>,
    /// Sort field
    pub sort: IssueSort,
    /// Sort direction
    pub direction: SortDirection,
    /// Results per page
    pub per_page: usize,
    /// Page number
    pub page: usize,
}

impl IssueParams {
    /// Create params for fetching all issues
    pub fn all() -> Self {
        Self {
            state: IssueStateFilter::All,
            per_page: 100,
            ..Default::default()
        }
    }

    /// Create params for fetching open issues
    pub fn open() -> Self {
        Self {
            state: IssueStateFilter::Open,
            per_page: 100,
            ..Default::default()
        }
    }

    /// Create params for fetching closed issues
    pub fn closed() -> Self {
        Self {
            state: IssueStateFilter::Closed,
            per_page: 100,
            ..Default::default()
        }
    }

    /// Set labels filter
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Set assignee filter
    pub fn with_assignee(mut self, assignee: impl Into<String>) -> Self {
        self.assignee = Some(assignee.into());
        self
    }

    /// Set milestone filter
    pub fn with_milestone(mut self, milestone: impl Into<String>) -> Self {
        self.milestone = Some(milestone.into());
        self
    }

    /// Set since filter
    pub fn since(mut self, since: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self
    }
}

/// Parameters for listing pull requests
#[derive(Debug, Clone, Default)]
pub struct PullParams {
    /// Filter by state
    pub state: PullStateFilter,
    /// Filter by head branch
    pub head: Option<String>,
    /// Filter by base branch
    pub base: Option<String>,
    /// Sort field
    pub sort: IssueSort,
    /// Sort direction
    pub direction: SortDirection,
    /// Results per page
    pub per_page: usize,
    /// Page number
    pub page: usize,
}

impl PullParams {
    /// Create params for fetching all PRs
    pub fn all() -> Self {
        Self {
            state: PullStateFilter::All,
            per_page: 100,
            ..Default::default()
        }
    }

    /// Create params for fetching open PRs
    pub fn open() -> Self {
        Self {
            state: PullStateFilter::Open,
            per_page: 100,
            ..Default::default()
        }
    }

    /// Create params for fetching merged PRs
    pub fn merged() -> Self {
        Self {
            state: PullStateFilter::Closed,
            per_page: 100,
            ..Default::default()
        }
    }
}
