//! Calendar event model types for change management calendar

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of calendar event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarEventType {
    /// Issue was created
    IssueCreated,
    /// Issue was closed
    IssueClosed,
    /// Milestone due date
    MilestoneDue,
    /// Milestone was closed
    MilestoneClosed,
    /// Release was published
    ReleasePublished,
    /// Pull request was merged
    PrMerged,
}

/// A single calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Unique event identifier
    pub id: String,
    /// Event type
    pub event_type: CalendarEventType,
    /// Event title
    pub title: String,
    /// Event date
    pub date: DateTime<Utc>,
    /// Issue/PR/milestone number
    pub number: Option<u64>,
    /// State (open, closed, etc.)
    pub state: Option<String>,
    /// Repository full name (owner/repo)
    pub repository: String,
    /// Labels associated with the event
    pub labels: Vec<String>,
    /// Associated milestone title
    pub milestone: Option<String>,
    /// URL to the event on GitHub
    pub url: String,
}

/// Summary of calendar events by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSummary {
    /// Total number of events
    pub total_events: usize,
    /// Event count by type
    pub by_type: HashMap<String, usize>,
}

/// Calendar data for a date range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarData {
    /// Start of date range
    pub start_date: NaiveDate,
    /// End of date range
    pub end_date: NaiveDate,
    /// Events in the date range
    pub events: Vec<CalendarEvent>,
    /// Summary statistics
    pub summary: CalendarSummary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_calendar_event_type_serialization() {
        let cases = vec![
            (CalendarEventType::IssueCreated, "\"issue_created\""),
            (CalendarEventType::IssueClosed, "\"issue_closed\""),
            (CalendarEventType::MilestoneDue, "\"milestone_due\""),
            (CalendarEventType::MilestoneClosed, "\"milestone_closed\""),
            (CalendarEventType::ReleasePublished, "\"release_published\""),
            (CalendarEventType::PrMerged, "\"pr_merged\""),
        ];

        for (variant, expected) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected, "Failed for {:?}", variant);
        }
    }

    #[test]
    fn test_calendar_event_type_deserialization() {
        let cases = vec![
            ("\"issue_created\"", CalendarEventType::IssueCreated),
            ("\"issue_closed\"", CalendarEventType::IssueClosed),
            ("\"milestone_due\"", CalendarEventType::MilestoneDue),
            ("\"milestone_closed\"", CalendarEventType::MilestoneClosed),
            ("\"release_published\"", CalendarEventType::ReleasePublished),
            ("\"pr_merged\"", CalendarEventType::PrMerged),
        ];

        for (json, expected) in cases {
            let parsed: CalendarEventType = serde_json::from_str(json).unwrap();
            assert_eq!(parsed, expected, "Failed for {}", json);
        }
    }

    #[test]
    fn test_calendar_event_type_roundtrip() {
        let variants = vec![
            CalendarEventType::IssueCreated,
            CalendarEventType::IssueClosed,
            CalendarEventType::MilestoneDue,
            CalendarEventType::MilestoneClosed,
            CalendarEventType::ReleasePublished,
            CalendarEventType::PrMerged,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: CalendarEventType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn test_calendar_summary_counts() {
        let summary = CalendarSummary {
            total_events: 5,
            by_type: HashMap::from([
                ("issue_created".to_string(), 2),
                ("release_published".to_string(), 3),
            ]),
        };

        assert_eq!(summary.total_events, 5);
        assert_eq!(summary.by_type["issue_created"], 2);
        assert_eq!(summary.by_type["release_published"], 3);
        assert_eq!(summary.by_type.get("pr_merged"), None);
    }

    #[test]
    fn test_calendar_data_date_range() {
        let data = CalendarData {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            events: vec![],
            summary: CalendarSummary {
                total_events: 0,
                by_type: HashMap::new(),
            },
        };

        assert_eq!(data.start_date.year(), 2026);
        assert_eq!(data.start_date.month(), 1);
        assert_eq!(data.end_date.month(), 3);
        assert_eq!(data.end_date.day(), 31);
    }

    #[test]
    fn test_calendar_event_construction() {
        let event = CalendarEvent {
            id: "test/repo-issue-created-42".to_string(),
            event_type: CalendarEventType::IssueCreated,
            title: "Fix login bug".to_string(),
            date: Utc::now(),
            number: Some(42),
            state: Some("open".to_string()),
            repository: "test/repo".to_string(),
            labels: vec!["bug".to_string()],
            milestone: Some("v1.0".to_string()),
            url: "https://github.com/test/repo/issues/42".to_string(),
        };

        assert_eq!(event.event_type, CalendarEventType::IssueCreated);
        assert_eq!(event.number, Some(42));
        assert_eq!(event.labels.len(), 1);
        assert_eq!(event.milestone, Some("v1.0".to_string()));
    }

    #[test]
    fn test_calendar_data_serialization_roundtrip() {
        let data = CalendarData {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            events: vec![CalendarEvent {
                id: "test-1".to_string(),
                event_type: CalendarEventType::PrMerged,
                title: "Add feature".to_string(),
                date: Utc::now(),
                number: Some(10),
                state: Some("merged".to_string()),
                repository: "owner/repo".to_string(),
                labels: vec![],
                milestone: None,
                url: "https://github.com/owner/repo/pull/10".to_string(),
            }],
            summary: CalendarSummary {
                total_events: 1,
                by_type: HashMap::from([("pr_merged".to_string(), 1)]),
            },
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: CalendarData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.summary.total_events, 1);
        assert_eq!(parsed.start_date, data.start_date);
        assert_eq!(parsed.end_date, data.end_date);
    }
}
