//! SLA (Service Level Agreement) metrics

use crate::config::SlaConfig;
use crate::models::{Issue, IssueEvent, IssueState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SLA compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaReport {
    /// Total issues evaluated
    pub total_issues: usize,
    /// Issues meeting response SLA
    pub response_sla_met: usize,
    /// Issues breaching response SLA
    pub response_sla_breached: usize,
    /// Issues meeting resolution SLA
    pub resolution_sla_met: usize,
    /// Issues breaching resolution SLA
    pub resolution_sla_breached: usize,
    /// Response SLA compliance percentage
    pub response_compliance_percent: f64,
    /// Resolution SLA compliance percentage
    pub resolution_compliance_percent: f64,
    /// List of violations
    pub violations: Vec<SlaViolation>,
}

/// SLA violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaViolation {
    /// Issue number
    pub issue_number: u64,
    /// Issue title
    pub issue_title: String,
    /// Type of violation
    pub violation_type: ViolationType,
    /// SLA target (hours)
    pub sla_hours: i64,
    /// Actual time (hours)
    pub actual_hours: i64,
    /// Time exceeded (hours)
    pub exceeded_by_hours: i64,
}

/// Type of SLA violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// First response time violation
    Response,
    /// Resolution time violation
    Resolution,
}

/// Calculator for SLA metrics
pub struct SlaCalculator {
    config: SlaConfig,
}

impl SlaCalculator {
    /// Create a new SLA calculator with the given config
    pub fn new(config: SlaConfig) -> Self {
        Self { config }
    }

    /// Calculate SLA compliance for issues
    pub fn calculate(
        &self,
        issues: &[Issue],
        events: &HashMap<u64, Vec<IssueEvent>>,
    ) -> SlaReport {
        let mut response_met = 0;
        let mut response_breached = 0;
        let mut resolution_met = 0;
        let mut resolution_breached = 0;
        let mut violations = Vec::new();

        for issue in issues {
            let (response_hours, resolution_hours) = self.get_sla_for_issue(issue);

            // Check response SLA
            if let Some(issue_events) = events.get(&issue.number) {
                let first_response = issue_events
                    .iter()
                    .filter(|e| e.event_type == "commented")
                    .min_by_key(|e| e.created_at);

                if let Some(response) = first_response {
                    let response_time = (response.created_at - issue.created_at).num_hours();
                    if response_time <= response_hours {
                        response_met += 1;
                    } else {
                        response_breached += 1;
                        violations.push(SlaViolation {
                            issue_number: issue.number,
                            issue_title: issue.title.clone(),
                            violation_type: ViolationType::Response,
                            sla_hours: response_hours,
                            actual_hours: response_time,
                            exceeded_by_hours: response_time - response_hours,
                        });
                    }
                }
            }

            // Check resolution SLA for closed issues
            if issue.state == IssueState::Closed {
                if let Some(closed_at) = issue.closed_at {
                    let resolution_time = (closed_at - issue.created_at).num_hours();
                    if resolution_time <= resolution_hours {
                        resolution_met += 1;
                    } else {
                        resolution_breached += 1;
                        violations.push(SlaViolation {
                            issue_number: issue.number,
                            issue_title: issue.title.clone(),
                            violation_type: ViolationType::Resolution,
                            sla_hours: resolution_hours,
                            actual_hours: resolution_time,
                            exceeded_by_hours: resolution_time - resolution_hours,
                        });
                    }
                }
            }
        }

        let total_with_response = response_met + response_breached;
        let total_closed = resolution_met + resolution_breached;

        SlaReport {
            total_issues: issues.len(),
            response_sla_met: response_met,
            response_sla_breached: response_breached,
            resolution_sla_met: resolution_met,
            resolution_sla_breached: resolution_breached,
            response_compliance_percent: if total_with_response > 0 {
                response_met as f64 / total_with_response as f64 * 100.0
            } else {
                100.0
            },
            resolution_compliance_percent: if total_closed > 0 {
                resolution_met as f64 / total_closed as f64 * 100.0
            } else {
                100.0
            },
            violations,
        }
    }

    fn get_sla_for_issue(&self, issue: &Issue) -> (i64, i64) {
        // Check for priority label
        for label in &issue.labels {
            let lower = label.name.to_lowercase();
            if let Some(priority) = self.config.priority.get(&lower) {
                return (priority.response_time_hours, priority.resolution_time_hours);
            }
        }

        (
            self.config.response_time_hours,
            self.config.resolution_time_hours,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SlaPriority;
    use crate::models::{Label, User};
    use chrono::{Duration, Utc};

    fn create_test_user() -> User {
        User {
            id: 1,
            login: "test".to_string(),
            avatar_url: "".to_string(),
            html_url: "".to_string(),
        }
    }

    fn create_test_issue(number: u64, created_hours_ago: i64, closed_hours_ago: Option<i64>, labels: Vec<Label>) -> Issue {
        let now = Utc::now();
        Issue {
            id: number as i64,
            number,
            title: format!("Issue #{}", number),
            body: None,
            state: if closed_hours_ago.is_some() { IssueState::Closed } else { IssueState::Open },
            labels,
            assignees: vec![],
            milestone: None,
            author: create_test_user(),
            comments_count: 0,
            created_at: now - Duration::hours(created_hours_ago),
            updated_at: now,
            closed_at: closed_hours_ago.map(|h| now - Duration::hours(h)),
            closed_by: None,
        }
    }

    fn default_sla_config() -> SlaConfig {
        SlaConfig {
            response_time_hours: 24,
            resolution_time_hours: 168,
            priority: HashMap::new(),
        }
    }

    #[test]
    fn test_sla_calculation_response_met() {
        let config = default_sla_config();
        let calculator = SlaCalculator::new(config);

        let now = Utc::now();
        let issue = create_test_issue(1, 48, None, vec![]);

        // Create event showing response within SLA (12 hours after creation)
        let events: HashMap<u64, Vec<IssueEvent>> = [(
            1,
            vec![IssueEvent {
                id: 1,
                event_type: "commented".to_string(),
                actor: Some(create_test_user()),
                created_at: now - Duration::hours(36), // 12 hours after issue creation
                label_name: None,
                assignee: None,
            }],
        )]
        .into();

        let report = calculator.calculate(&[issue], &events);
        assert_eq!(report.response_sla_met, 1);
        assert_eq!(report.response_sla_breached, 0);
    }

    #[test]
    fn test_sla_calculation_response_breached() {
        let config = default_sla_config();
        let calculator = SlaCalculator::new(config);

        let now = Utc::now();
        let issue = create_test_issue(1, 72, None, vec![]);

        // Create event showing response outside SLA (48 hours after creation)
        let events: HashMap<u64, Vec<IssueEvent>> = [(
            1,
            vec![IssueEvent {
                id: 1,
                event_type: "commented".to_string(),
                actor: Some(create_test_user()),
                created_at: now - Duration::hours(24), // 48 hours after issue creation (breaches 24h SLA)
                label_name: None,
                assignee: None,
            }],
        )]
        .into();

        let report = calculator.calculate(&[issue], &events);
        assert_eq!(report.response_sla_met, 0);
        assert_eq!(report.response_sla_breached, 1);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].violation_type, ViolationType::Response);
    }

    #[test]
    fn test_sla_calculation_resolution_met() {
        let config = default_sla_config();
        let calculator = SlaCalculator::new(config);

        // Issue created 100 hours ago, closed 4 hours ago (96 hour resolution - within 168h)
        let issue = create_test_issue(1, 100, Some(4), vec![]);
        let events: HashMap<u64, Vec<IssueEvent>> = HashMap::new();

        let report = calculator.calculate(&[issue], &events);
        assert_eq!(report.resolution_sla_met, 1);
        assert_eq!(report.resolution_sla_breached, 0);
    }

    #[test]
    fn test_sla_calculation_resolution_breached() {
        let config = default_sla_config();
        let calculator = SlaCalculator::new(config);

        // Issue created 200 hours ago, closed 4 hours ago (196 hour resolution - breaches 168h)
        let issue = create_test_issue(1, 200, Some(4), vec![]);
        let events: HashMap<u64, Vec<IssueEvent>> = HashMap::new();

        let report = calculator.calculate(&[issue], &events);
        assert_eq!(report.resolution_sla_met, 0);
        assert_eq!(report.resolution_sla_breached, 1);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].violation_type, ViolationType::Resolution);
    }

    #[test]
    fn test_sla_with_priority_labels() {
        let mut priority = HashMap::new();
        priority.insert(
            "critical".to_string(),
            SlaPriority {
                response_time_hours: 4,
                resolution_time_hours: 24,
            },
        );

        let config = SlaConfig {
            response_time_hours: 24,
            resolution_time_hours: 168,
            priority,
        };
        let calculator = SlaCalculator::new(config);

        let critical_label = Label {
            id: 1,
            name: "critical".to_string(),
            color: "ff0000".to_string(),
            description: None,
        };

        // Issue with critical label, created 30 hours ago, closed 5 hours ago (25h resolution)
        // This breaches the 24h critical SLA
        let issue = create_test_issue(1, 30, Some(5), vec![critical_label]);
        let events: HashMap<u64, Vec<IssueEvent>> = HashMap::new();

        let report = calculator.calculate(&[issue], &events);
        assert_eq!(report.resolution_sla_breached, 1);
        assert!(report.violations.iter().any(|v| v.sla_hours == 24));
    }

    #[test]
    fn test_sla_compliance_percentage() {
        let config = default_sla_config();
        let calculator = SlaCalculator::new(config);

        // 2 issues closed within SLA, 1 breached
        let issues = vec![
            create_test_issue(1, 100, Some(4), vec![]),   // Within SLA (96h)
            create_test_issue(2, 120, Some(10), vec![]),  // Within SLA (110h)
            create_test_issue(3, 200, Some(5), vec![]),   // Breached (195h)
        ];
        let events: HashMap<u64, Vec<IssueEvent>> = HashMap::new();

        let report = calculator.calculate(&issues, &events);
        assert_eq!(report.resolution_sla_met, 2);
        assert_eq!(report.resolution_sla_breached, 1);
        assert!((report.resolution_compliance_percent - 66.67).abs() < 0.1);
    }
}
