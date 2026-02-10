//! SLA (Service Level Agreement) route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::convert;
use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use greport_core::client::{GitHubClient, IssueParams, RepoId};
use greport_core::models::{Issue, IssueState};

#[derive(Deserialize)]
pub struct SlaQuery {
    /// Custom response time SLA in hours
    response_hours: Option<i64>,
    /// Custom resolution time SLA in hours
    resolution_hours: Option<i64>,
    /// Filter by labels (comma-separated)
    labels: Option<String>,
}

/// SLA compliance report
#[derive(Serialize)]
pub struct SlaReport {
    /// Repository identifier
    pub repository: String,
    /// SLA configuration used
    pub config: SlaConfig,
    /// Summary statistics
    pub summary: SlaSummary,
    /// Issues breaching SLA
    pub breaching_issues: Vec<SlaIssue>,
    /// Issues at risk of breaching
    pub at_risk_issues: Vec<SlaIssue>,
    /// Generated at timestamp
    pub generated_at: String,
}

/// SLA configuration
#[derive(Serialize)]
pub struct SlaConfig {
    pub response_time_hours: i64,
    pub resolution_time_hours: i64,
}

/// SLA summary statistics
#[derive(Serialize)]
pub struct SlaSummary {
    /// Total open issues
    pub total_open: usize,
    /// Issues within SLA
    pub within_sla: usize,
    /// Issues breaching response SLA
    pub response_breached: usize,
    /// Issues breaching resolution SLA
    pub resolution_breached: usize,
    /// Issues at risk (>80% of SLA time elapsed)
    pub at_risk: usize,
    /// SLA compliance percentage
    pub compliance_rate: f64,
}

/// Issue with SLA status
#[derive(Serialize)]
pub struct SlaIssue {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub created_at: String,
    pub age_hours: i64,
    pub sla_status: SlaStatus,
    pub labels: Vec<String>,
}

/// SLA status for an issue
#[derive(Serialize)]
pub enum SlaStatus {
    /// Within SLA limits
    Ok,
    /// At risk (>80% of time elapsed)
    AtRisk { percent_elapsed: f64 },
    /// Response time breached
    ResponseBreached { hours_overdue: i64 },
    /// Resolution time breached
    ResolutionBreached { hours_overdue: i64 },
}

pub async fn get_sla_report(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<SlaQuery>,
) -> Result<Json<ApiResponse<SlaReport>>, ApiError> {
    // Get SLA thresholds
    let response_hours = query
        .response_hours
        .unwrap_or(state.config.sla_response_hours);
    let resolution_hours = query
        .resolution_hours
        .unwrap_or(state.config.sla_resolution_hours);

    // DB-first
    let issues = if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "issues").await {
                convert::issues_from_db(pool, repo_db_id, Some("open"), None).await?
            } else {
                fetch_open_issues(&state, &owner, &repo, &query).await?
            }
        } else {
            fetch_open_issues(&state, &owner, &repo, &query).await?
        }
    } else {
        fetch_open_issues(&state, &owner, &repo, &query).await?
    };

    let report = build_sla_report(&owner, &repo, &issues, response_hours, resolution_hours);

    Ok(Json(ApiResponse::ok(report)))
}

async fn fetch_open_issues(
    state: &AppState,
    owner: &str,
    repo: &str,
    query: &SlaQuery,
) -> Result<Vec<Issue>, ApiError> {
    let repo_id = RepoId::new(owner.to_string(), repo.to_string());
    let params = IssueParams {
        labels: query
            .labels
            .as_ref()
            .map(|l| l.split(',').map(String::from).collect()),
        ..IssueParams::open()
    };
    let client = state.client_for_owner(owner)?;
    let issues = client.list_issues(&repo_id, params).await?;
    Ok(issues)
}

fn build_sla_report(
    owner: &str,
    repo: &str,
    issues: &[Issue],
    response_hours: i64,
    resolution_hours: i64,
) -> SlaReport {
    let now = Utc::now();
    let response_threshold = Duration::hours(response_hours);
    let resolution_threshold = Duration::hours(resolution_hours);
    let at_risk_threshold = 0.8; // 80%

    let mut breaching_issues = Vec::new();
    let mut at_risk_issues = Vec::new();
    let mut response_breached = 0;
    let mut resolution_breached = 0;
    let mut at_risk_count = 0;
    let mut within_sla = 0;

    for issue in issues {
        if issue.state != IssueState::Open {
            continue;
        }

        let age = now.signed_duration_since(issue.created_at);
        let age_hours = age.num_hours();

        let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();
        let url = format!(
            "https://github.com/{}/{}/issues/{}",
            owner, repo, issue.number
        );

        let sla_status = if age > resolution_threshold {
            resolution_breached += 1;
            SlaStatus::ResolutionBreached {
                hours_overdue: age_hours - resolution_hours,
            }
        } else if age > response_threshold {
            // Check if there's been any response (comments > 0)
            if issue.comments_count == 0 {
                response_breached += 1;
                SlaStatus::ResponseBreached {
                    hours_overdue: age_hours - response_hours,
                }
            } else {
                // Has response, check resolution SLA
                let percent = age_hours as f64 / resolution_hours as f64;
                if percent >= at_risk_threshold {
                    at_risk_count += 1;
                    SlaStatus::AtRisk {
                        percent_elapsed: percent * 100.0,
                    }
                } else {
                    within_sla += 1;
                    SlaStatus::Ok
                }
            }
        } else {
            // Check if at risk
            let percent = age_hours as f64 / response_hours as f64;
            if percent >= at_risk_threshold && issue.comments_count == 0 {
                at_risk_count += 1;
                SlaStatus::AtRisk {
                    percent_elapsed: percent * 100.0,
                }
            } else {
                within_sla += 1;
                SlaStatus::Ok
            }
        };

        let sla_issue = SlaIssue {
            number: issue.number,
            title: issue.title.clone(),
            url,
            author: issue.author.login.clone(),
            created_at: issue.created_at.to_rfc3339(),
            age_hours,
            sla_status: sla_status.clone(),
            labels,
        };

        match sla_status {
            SlaStatus::ResponseBreached { .. } | SlaStatus::ResolutionBreached { .. } => {
                breaching_issues.push(sla_issue);
            }
            SlaStatus::AtRisk { .. } => {
                at_risk_issues.push(sla_issue);
            }
            SlaStatus::Ok => {}
        }
    }

    // Sort by age (oldest first)
    breaching_issues.sort_by(|a, b| b.age_hours.cmp(&a.age_hours));
    at_risk_issues.sort_by(|a, b| b.age_hours.cmp(&a.age_hours));

    let total_open = issues.len();
    let compliance_rate = if total_open > 0 {
        (within_sla as f64 / total_open as f64) * 100.0
    } else {
        100.0
    };

    SlaReport {
        repository: format!("{}/{}", owner, repo),
        config: SlaConfig {
            response_time_hours: response_hours,
            resolution_time_hours: resolution_hours,
        },
        summary: SlaSummary {
            total_open,
            within_sla,
            response_breached,
            resolution_breached,
            at_risk: at_risk_count,
            compliance_rate,
        },
        breaching_issues,
        at_risk_issues,
        generated_at: now.to_rfc3339(),
    }
}

impl Clone for SlaStatus {
    fn clone(&self) -> Self {
        match self {
            SlaStatus::Ok => SlaStatus::Ok,
            SlaStatus::AtRisk { percent_elapsed } => SlaStatus::AtRisk {
                percent_elapsed: *percent_elapsed,
            },
            SlaStatus::ResponseBreached { hours_overdue } => SlaStatus::ResponseBreached {
                hours_overdue: *hours_overdue,
            },
            SlaStatus::ResolutionBreached { hours_overdue } => SlaStatus::ResolutionBreached {
                hours_overdue: *hours_overdue,
            },
        }
    }
}
