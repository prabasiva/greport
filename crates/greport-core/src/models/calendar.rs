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
