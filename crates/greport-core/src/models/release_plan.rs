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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MilestoneState;
    use chrono::Duration;

    fn test_milestone(title: &str) -> Milestone {
        Milestone {
            id: 1,
            number: 1,
            title: title.to_string(),
            description: None,
            state: MilestoneState::Open,
            open_issues: 5,
            closed_issues: 15,
            due_on: Some(Utc::now() + Duration::days(30)),
            created_at: Utc::now() - Duration::days(60),
            closed_at: None,
        }
    }

    fn test_release(tag: &str, draft: bool, prerelease: bool) -> Release {
        Release {
            id: 1,
            tag_name: tag.to_string(),
            name: Some(format!("Release {}", tag)),
            body: Some("Release notes".to_string()),
            draft,
            prerelease,
            author: super::super::User::unknown(),
            created_at: Utc::now(),
            published_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_release_plan_status_serialization() {
        let cases = vec![
            (ReleasePlanStatus::OnTrack, "\"on_track\""),
            (ReleasePlanStatus::AtRisk, "\"at_risk\""),
            (ReleasePlanStatus::Overdue, "\"overdue\""),
        ];

        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected, "Failed for {:?}", variant);
        }
    }

    #[test]
    fn test_release_plan_status_deserialization() {
        let cases = vec![
            ("\"on_track\"", ReleasePlanStatus::OnTrack),
            ("\"at_risk\"", ReleasePlanStatus::AtRisk),
            ("\"overdue\"", ReleasePlanStatus::Overdue),
        ];

        for (json, expected) in cases {
            let parsed: ReleasePlanStatus = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected, "Failed for {}", json);
        }
    }

    #[test]
    fn test_release_plan_status_roundtrip() {
        let variants = vec![
            ReleasePlanStatus::OnTrack,
            ReleasePlanStatus::AtRisk,
            ReleasePlanStatus::Overdue,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: ReleasePlanStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn test_upcoming_release_construction() {
        let ms = test_milestone("v2.0");
        let upcoming = UpcomingRelease {
            milestone: ms,
            repository: "owner/repo".to_string(),
            progress_percent: 75.0,
            days_remaining: 30,
            blocker_count: 2,
            status: ReleasePlanStatus::OnTrack,
        };

        assert_eq!(upcoming.milestone.title, "v2.0");
        assert_eq!(upcoming.repository, "owner/repo");
        assert_eq!(upcoming.progress_percent, 75.0);
        assert_eq!(upcoming.days_remaining, 30);
        assert_eq!(upcoming.blocker_count, 2);
        assert_eq!(upcoming.status, ReleasePlanStatus::OnTrack);
    }

    #[test]
    fn test_recent_release_stable() {
        let release = test_release("v1.0.0", false, false);
        let recent = RecentRelease {
            release,
            repository: "owner/repo".to_string(),
            release_type: "stable".to_string(),
        };

        assert_eq!(recent.release_type, "stable");
        assert_eq!(recent.release.tag_name, "v1.0.0");
    }

    #[test]
    fn test_recent_release_prerelease() {
        let release = test_release("v2.0.0-beta.1", false, true);
        let recent = RecentRelease {
            release,
            repository: "owner/repo".to_string(),
            release_type: "prerelease".to_string(),
        };

        assert_eq!(recent.release_type, "prerelease");
        assert!(recent.release.prerelease);
    }

    #[test]
    fn test_recent_release_draft() {
        let release = test_release("v3.0.0", true, false);
        let recent = RecentRelease {
            release,
            repository: "owner/repo".to_string(),
            release_type: "draft".to_string(),
        };

        assert_eq!(recent.release_type, "draft");
        assert!(recent.release.draft);
    }

    #[test]
    fn test_timeline_entry_is_future() {
        let future_entry = TimelineEntry {
            date: Utc::now() + Duration::days(30),
            entry_type: "milestone".to_string(),
            title: "v2.0".to_string(),
            repository: "owner/repo".to_string(),
            is_future: true,
            progress_percent: Some(50.0),
        };

        let past_entry = TimelineEntry {
            date: Utc::now() - Duration::days(10),
            entry_type: "release".to_string(),
            title: "v1.0".to_string(),
            repository: "owner/repo".to_string(),
            is_future: false,
            progress_percent: None,
        };

        assert!(future_entry.is_future);
        assert!(!past_entry.is_future);
        assert_eq!(future_entry.progress_percent, Some(50.0));
        assert_eq!(past_entry.progress_percent, None);
    }

    #[test]
    fn test_release_plan_construction() {
        let plan = ReleasePlan {
            upcoming: vec![UpcomingRelease {
                milestone: test_milestone("v2.0"),
                repository: "owner/repo".to_string(),
                progress_percent: 60.0,
                days_remaining: 15,
                blocker_count: 1,
                status: ReleasePlanStatus::OnTrack,
            }],
            recent_releases: vec![RecentRelease {
                release: test_release("v1.5.0", false, false),
                repository: "owner/repo".to_string(),
                release_type: "stable".to_string(),
            }],
            timeline: vec![],
        };

        assert_eq!(plan.upcoming.len(), 1);
        assert_eq!(plan.recent_releases.len(), 1);
        assert_eq!(plan.timeline.len(), 0);
    }

    #[test]
    fn test_release_plan_serialization_roundtrip() {
        let plan = ReleasePlan {
            upcoming: vec![],
            recent_releases: vec![],
            timeline: vec![TimelineEntry {
                date: Utc::now(),
                entry_type: "release".to_string(),
                title: "v1.0.0".to_string(),
                repository: "owner/repo".to_string(),
                is_future: false,
                progress_percent: None,
            }],
        };

        let json = serde_json::to_string(&plan).unwrap();
        let parsed: ReleasePlan = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.timeline.len(), 1);
        assert_eq!(parsed.timeline[0].title, "v1.0.0");
    }
}
