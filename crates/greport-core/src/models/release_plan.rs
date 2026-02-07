//! Release management plan model types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Milestone, Release};

/// Status of an upcoming release milestone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePlanStatus {
    /// Milestone is progressing normally
    OnTrack,
    /// Less than 25% time remaining with less than 75% issues closed
    AtRisk,
    /// Due date has passed and milestone is still open
    Overdue,
}

/// An upcoming release (open milestone with due date)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingRelease {
    /// The milestone
    pub milestone: Milestone,
    /// Repository full name
    pub repository: String,
    /// Completion percentage (0.0 - 100.0)
    pub progress_percent: f64,
    /// Days until due date (negative if overdue)
    pub days_remaining: i64,
    /// Count of issues with blocker/critical labels
    pub blocker_count: usize,
    /// Computed status
    pub status: ReleasePlanStatus,
}

/// A recently published release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRelease {
    /// The release
    pub release: Release,
    /// Repository full name
    pub repository: String,
    /// Release type classification
    pub release_type: String,
}

/// A timeline entry (milestone or release plotted on timeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Date of the event
    pub date: DateTime<Utc>,
    /// Entry type: "release" or "milestone"
    pub entry_type: String,
    /// Display title
    pub title: String,
    /// Repository full name
    pub repository: String,
    /// Whether this is a future event
    pub is_future: bool,
    /// Progress percentage (milestones only)
    pub progress_percent: Option<f64>,
}

/// Complete release plan data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePlan {
    /// Upcoming milestones sorted by due date
    pub upcoming: Vec<UpcomingRelease>,
    /// Recently published releases
    pub recent_releases: Vec<RecentRelease>,
    /// Combined timeline of milestones and releases
    pub timeline: Vec<TimelineEntry>,
}
