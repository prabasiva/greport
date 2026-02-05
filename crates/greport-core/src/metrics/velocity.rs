//! Velocity metrics calculations

use crate::models::{Issue, IssueState};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Time period for velocity calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    /// Daily
    Day,
    /// Weekly
    Week,
    /// Monthly
    Month,
}

impl Period {
    /// Get duration for this period
    pub fn duration(&self) -> Duration {
        match self {
            Period::Day => Duration::days(1),
            Period::Week => Duration::weeks(1),
            Period::Month => Duration::days(30),
        }
    }

    /// Get display name
    pub fn label(&self) -> &'static str {
        match self {
            Period::Day => "day",
            Period::Week => "week",
            Period::Month => "month",
        }
    }
}

impl std::str::FromStr for Period {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" | "daily" => Ok(Period::Day),
            "week" | "weekly" => Ok(Period::Week),
            "month" | "monthly" => Ok(Period::Month),
            _ => Err(crate::Error::Custom(format!("Invalid period: {}", s))),
        }
    }
}

/// Velocity trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    /// Issue count is increasing
    Increasing,
    /// Issue count is decreasing
    Decreasing,
    /// Issue count is stable
    Stable,
}

impl Trend {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Trend::Increasing => "increasing",
            Trend::Decreasing => "decreasing",
            Trend::Stable => "stable",
        }
    }
}

/// Velocity metrics over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityMetrics {
    /// Time period used
    pub period: Period,
    /// Data points for each period
    pub data_points: Vec<VelocityDataPoint>,
    /// Average issues opened per period
    pub avg_opened: f64,
    /// Average issues closed per period
    pub avg_closed: f64,
    /// Overall trend
    pub trend: Trend,
}

/// Single velocity data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityDataPoint {
    /// Period start date
    pub period_start: DateTime<Utc>,
    /// Period end date
    pub period_end: DateTime<Utc>,
    /// Issues opened in this period
    pub opened: usize,
    /// Issues closed in this period
    pub closed: usize,
    /// Net change (opened - closed)
    pub net_change: i64,
    /// Cumulative open issues at end of period
    pub cumulative_open: usize,
}

/// Calculator for velocity metrics
pub struct VelocityCalculator;

impl VelocityCalculator {
    /// Calculate velocity metrics for issues over a number of periods
    pub fn calculate(issues: &[Issue], period: Period, num_periods: usize) -> VelocityMetrics {
        let now = Utc::now();
        let period_duration = period.duration();

        let mut data_points = Vec::with_capacity(num_periods);

        // Calculate current open count
        let current_open = issues
            .iter()
            .filter(|i| i.state == IssueState::Open)
            .count();

        // Work backwards from now
        let mut cumulative_open = current_open;

        for i in 0..num_periods {
            let period_end = now - period_duration * i as i32;
            let period_start = period_end - period_duration;

            let opened = issues
                .iter()
                .filter(|issue| issue.created_at >= period_start && issue.created_at < period_end)
                .count();

            let closed = issues
                .iter()
                .filter(|issue| {
                    issue
                        .closed_at
                        .map(|c| c >= period_start && c < period_end)
                        .unwrap_or(false)
                })
                .count();

            let net_change = opened as i64 - closed as i64;

            data_points.push(VelocityDataPoint {
                period_start,
                period_end,
                opened,
                closed,
                net_change,
                cumulative_open,
            });

            // Adjust cumulative for next (earlier) period
            cumulative_open = (cumulative_open as i64 - net_change).max(0) as usize;
        }

        // Reverse to chronological order
        data_points.reverse();

        let avg_opened = if data_points.is_empty() {
            0.0
        } else {
            data_points.iter().map(|d| d.opened).sum::<usize>() as f64 / data_points.len() as f64
        };

        let avg_closed = if data_points.is_empty() {
            0.0
        } else {
            data_points.iter().map(|d| d.closed).sum::<usize>() as f64 / data_points.len() as f64
        };

        let trend = Self::calculate_trend(&data_points);

        VelocityMetrics {
            period,
            data_points,
            avg_opened,
            avg_closed,
            trend,
        }
    }

    fn calculate_trend(data_points: &[VelocityDataPoint]) -> Trend {
        if data_points.len() < 4 {
            return Trend::Stable;
        }

        let mid = data_points.len() / 2;
        let first_half_net: i64 = data_points[..mid].iter().map(|d| d.net_change).sum();
        let second_half_net: i64 = data_points[mid..].iter().map(|d| d.net_change).sum();

        let threshold = 5;
        if second_half_net > first_half_net + threshold {
            Trend::Increasing
        } else if second_half_net < first_half_net - threshold {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    fn create_issue(created_days_ago: i64, closed_days_ago: Option<i64>) -> Issue {
        let created = Utc::now() - Duration::days(created_days_ago);
        Issue {
            id: 1,
            number: 1,
            title: "Test".to_string(),
            body: None,
            state: if closed_days_ago.is_some() {
                IssueState::Closed
            } else {
                IssueState::Open
            },
            labels: vec![],
            assignees: vec![],
            milestone: None,
            author: User::unknown(),
            comments_count: 0,
            created_at: created,
            updated_at: created,
            closed_at: closed_days_ago.map(|d| Utc::now() - Duration::days(d)),
            closed_by: None,
        }
    }

    #[test]
    fn test_velocity_calculation() {
        let issues = vec![
            create_issue(5, None),
            create_issue(10, Some(3)),
            create_issue(15, Some(12)),
        ];

        let velocity = VelocityCalculator::calculate(&issues, Period::Week, 4);

        assert_eq!(velocity.period, Period::Week);
        assert_eq!(velocity.data_points.len(), 4);
    }

    #[test]
    fn test_trend_stable() {
        let data_points = vec![
            VelocityDataPoint {
                period_start: Utc::now(),
                period_end: Utc::now(),
                opened: 5,
                closed: 5,
                net_change: 0,
                cumulative_open: 10,
            };
            8
        ];

        let trend = VelocityCalculator::calculate_trend(&data_points);
        assert_eq!(trend, Trend::Stable);
    }
}
