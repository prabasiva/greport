//! Cross-repository aggregate metrics route handlers

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::convert;
use crate::error::ApiError;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;
use greport_core::metrics::{
    IssueMetricsCalculator, Period, PullMetricsCalculator, VelocityCalculator,
};
use greport_core::models::{Issue, PullRequest};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct RepoIssueMetrics {
    pub repository: String,
    pub total: usize,
    pub open: usize,
    pub closed: usize,
    pub avg_time_to_close_hours: Option<f64>,
    pub stale_count: usize,
}

#[derive(Serialize)]
pub struct IssueMetricsTotals {
    pub total: usize,
    pub open: usize,
    pub closed: usize,
    pub avg_time_to_close_hours: Option<f64>,
    pub stale_count: usize,
}

#[derive(Serialize)]
pub struct AgeBucketResponse {
    pub label: String,
    pub min_days: i64,
    pub max_days: Option<i64>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct AggregateIssueMetrics {
    pub by_repository: Vec<RepoIssueMetrics>,
    pub totals: IssueMetricsTotals,
    pub by_label: HashMap<String, usize>,
    pub by_assignee: HashMap<String, usize>,
    pub age_distribution: Vec<AgeBucketResponse>,
}

#[derive(Serialize)]
pub struct RepoPullMetrics {
    pub repository: String,
    pub total: usize,
    pub open: usize,
    pub merged: usize,
    pub avg_time_to_merge_hours: Option<f64>,
}

#[derive(Serialize)]
pub struct PullMetricsTotals {
    pub total: usize,
    pub open: usize,
    pub merged: usize,
    pub avg_time_to_merge_hours: Option<f64>,
}

#[derive(Serialize)]
pub struct AggregatePullMetrics {
    pub by_repository: Vec<RepoPullMetrics>,
    pub totals: PullMetricsTotals,
    pub by_size: HashMap<String, usize>,
    pub by_author: HashMap<String, usize>,
}

#[derive(Serialize)]
pub struct AggregateContributorStats {
    pub login: String,
    pub repositories: Vec<String>,
    pub total_issues_created: usize,
    pub total_prs_created: usize,
    pub total_prs_merged: usize,
}

#[derive(Serialize)]
pub struct RepoVelocityEntry {
    pub repository: String,
    pub avg_opened: f64,
    pub avg_closed: f64,
}

#[derive(Serialize)]
pub struct AggregateVelocityMetrics {
    pub period: String,
    pub by_repository: Vec<RepoVelocityEntry>,
    pub combined_avg_opened: f64,
    pub combined_avg_closed: f64,
    pub trend: String,
}

#[derive(Deserialize)]
pub struct IssueMetricsQuery {
    state: Option<String>,
    days: Option<i64>,
}

#[derive(Deserialize)]
pub struct PullMetricsQuery {
    state: Option<String>,
    days: Option<i64>,
}

#[derive(Deserialize)]
pub struct AggregateListQuery {
    state: Option<String>,
    days: Option<i64>,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Serialize)]
pub struct AggregateIssueItem {
    pub repository: String,
    #[serde(flatten)]
    pub issue: Issue,
}

#[derive(Serialize)]
pub struct AggregatePullItem {
    pub repository: String,
    #[serde(flatten)]
    pub pull: PullRequest,
}

#[derive(Deserialize)]
pub struct VelocityQuery {
    period: Option<String>,
    last: Option<usize>,
}

// ---------------------------------------------------------------------------
// Helper: get all synced repos
// ---------------------------------------------------------------------------

struct RepoData {
    full_name: String,
    db_id: i64,
}

async fn get_synced_repos(state: &AppState) -> Result<Vec<RepoData>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate metrics".into()))?;

    let tracked = greport_db::queries::list_tracked_repos(pool).await?;
    let mut repos = Vec::new();

    for tracked_repo in tracked {
        let parts: Vec<&str> = tracked_repo.full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            continue;
        }
        if let Some(db_id) = convert::get_repo_db_id(pool, parts[0], parts[1]).await {
            repos.push(RepoData {
                full_name: tracked_repo.full_name.clone(),
                db_id,
            });
        }
    }

    Ok(repos)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/aggregate/issues/metrics
pub async fn aggregate_issue_metrics(
    State(state): State<AppState>,
    Query(query): Query<IssueMetricsQuery>,
) -> Result<Json<ApiResponse<AggregateIssueMetrics>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate metrics".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;

    let mut all_issues: Vec<Issue> = Vec::new();
    let mut by_repository = Vec::new();
    let mut total_total = 0usize;
    let mut total_open = 0usize;
    let mut total_closed = 0usize;
    let mut total_stale = 0usize;
    let mut all_close_hours = Vec::new();

    for repo in &repos {
        let issues = convert::issues_from_db(pool, repo.db_id, None, None).await?;
        let filtered: Vec<Issue> = filter_issues(issues, state_filter, days_filter);

        let calc = IssueMetricsCalculator::default();
        let metrics = calc.calculate(&filtered);

        total_total += metrics.total;
        total_open += metrics.open;
        total_closed += metrics.closed;
        total_stale += metrics.stale_count;

        if let Some(avg) = metrics.avg_time_to_close_hours {
            for _ in 0..metrics.closed {
                all_close_hours.push(avg);
            }
        }

        by_repository.push(RepoIssueMetrics {
            repository: repo.full_name.clone(),
            total: metrics.total,
            open: metrics.open,
            closed: metrics.closed,
            avg_time_to_close_hours: metrics.avg_time_to_close_hours,
            stale_count: metrics.stale_count,
        });

        all_issues.extend(filtered);
    }

    let avg_close = if all_close_hours.is_empty() {
        None
    } else {
        Some(all_close_hours.iter().sum::<f64>() / all_close_hours.len() as f64)
    };

    // Combined distribution metrics across all repos
    let calc = IssueMetricsCalculator::default();
    let combined = calc.calculate(&all_issues);

    let age_distribution = combined
        .age_distribution
        .buckets
        .into_iter()
        .map(|b| AgeBucketResponse {
            label: b.label,
            min_days: b.min_days,
            max_days: b.max_days,
            count: b.count,
        })
        .collect();

    Ok(Json(ApiResponse::ok(AggregateIssueMetrics {
        by_repository,
        totals: IssueMetricsTotals {
            total: total_total,
            open: total_open,
            closed: total_closed,
            avg_time_to_close_hours: avg_close,
            stale_count: total_stale,
        },
        by_label: combined.by_label,
        by_assignee: combined.by_assignee,
        age_distribution,
    })))
}

fn filter_issues(issues: Vec<Issue>, state: Option<&str>, days: Option<i64>) -> Vec<Issue> {
    use greport_core::models::IssueState;
    let cutoff = days.map(|d| Utc::now() - chrono::Duration::days(d));
    issues
        .into_iter()
        .filter(|i| match state {
            Some("open") => i.state == IssueState::Open,
            Some("closed") => i.state == IssueState::Closed,
            _ => true,
        })
        .filter(|i| match cutoff {
            Some(c) => i.created_at >= c,
            None => true,
        })
        .collect()
}

fn filter_pulls(
    pulls: Vec<PullRequest>,
    state: Option<&str>,
    days: Option<i64>,
) -> Vec<PullRequest> {
    use greport_core::models::PullState;
    let cutoff = days.map(|d| Utc::now() - chrono::Duration::days(d));
    pulls
        .into_iter()
        .filter(|p| match state {
            Some("open") => p.state == PullState::Open,
            Some("closed") => p.state == PullState::Closed,
            _ => true,
        })
        .filter(|p| match cutoff {
            Some(c) => p.created_at >= c,
            None => true,
        })
        .collect()
}

/// GET /api/v1/aggregate/issues
pub async fn aggregate_issues_list(
    State(state): State<AppState>,
    Query(query): Query<AggregateListQuery>,
) -> Result<Json<PaginatedResponse<AggregateIssueItem>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate list".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;
    let mut all_items: Vec<AggregateIssueItem> = Vec::new();

    for repo in &repos {
        let issues = convert::issues_from_db(pool, repo.db_id, None, None).await?;
        let filtered = filter_issues(issues, state_filter, days_filter);
        for issue in filtered {
            all_items.push(AggregateIssueItem {
                repository: repo.full_name.clone(),
                issue,
            });
        }
    }

    // Sort newest first
    all_items.sort_by(|a, b| b.issue.created_at.cmp(&a.issue.created_at));

    let total = all_items.len() as u32;
    Ok(Json(PaginatedResponse::new(
        all_items,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

/// GET /api/v1/aggregate/pulls
pub async fn aggregate_pulls_list(
    State(state): State<AppState>,
    Query(query): Query<AggregateListQuery>,
) -> Result<Json<PaginatedResponse<AggregatePullItem>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate list".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;
    let mut all_items: Vec<AggregatePullItem> = Vec::new();

    for repo in &repos {
        let pulls = convert::pulls_from_db(pool, repo.db_id, None, None).await?;
        let filtered = filter_pulls(pulls, state_filter, days_filter);
        for pull in filtered {
            all_items.push(AggregatePullItem {
                repository: repo.full_name.clone(),
                pull,
            });
        }
    }

    // Sort newest first
    all_items.sort_by(|a, b| b.pull.created_at.cmp(&a.pull.created_at));

    let total = all_items.len() as u32;
    Ok(Json(PaginatedResponse::new(
        all_items,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

// ---------------------------------------------------------------------------
// Cross-org aggregate types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct OrgAggregateIssueItem {
    pub organization: String,
    pub repository: String,
    #[serde(flatten)]
    pub issue: Issue,
}

#[derive(Serialize)]
pub struct OrgAggregatePullItem {
    pub organization: String,
    pub repository: String,
    #[serde(flatten)]
    pub pull: PullRequest,
}

/// GET /api/v1/aggregate/orgs/issues
///
/// Cross-org aggregation: lists issues across all synced repos with org field.
pub async fn aggregate_org_issues(
    State(state): State<AppState>,
    Query(query): Query<AggregateListQuery>,
) -> Result<Json<PaginatedResponse<OrgAggregateIssueItem>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate list".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;
    let mut all_items: Vec<OrgAggregateIssueItem> = Vec::new();

    for repo in &repos {
        let issues = convert::issues_from_db(pool, repo.db_id, None, None).await?;
        let filtered = filter_issues(issues, state_filter, days_filter);
        let organization = repo.full_name.split('/').next().unwrap_or("").to_string();
        for issue in filtered {
            all_items.push(OrgAggregateIssueItem {
                organization: organization.clone(),
                repository: repo.full_name.clone(),
                issue,
            });
        }
    }

    // Sort newest first
    all_items.sort_by(|a, b| b.issue.created_at.cmp(&a.issue.created_at));

    let total = all_items.len() as u32;
    Ok(Json(PaginatedResponse::new(
        all_items,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

/// GET /api/v1/aggregate/orgs/pulls
///
/// Cross-org aggregation: lists pull requests across all synced repos with org field.
pub async fn aggregate_org_pulls(
    State(state): State<AppState>,
    Query(query): Query<AggregateListQuery>,
) -> Result<Json<PaginatedResponse<OrgAggregatePullItem>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate list".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;
    let mut all_items: Vec<OrgAggregatePullItem> = Vec::new();

    for repo in &repos {
        let pulls = convert::pulls_from_db(pool, repo.db_id, None, None).await?;
        let filtered = filter_pulls(pulls, state_filter, days_filter);
        let organization = repo.full_name.split('/').next().unwrap_or("").to_string();
        for pull in filtered {
            all_items.push(OrgAggregatePullItem {
                organization: organization.clone(),
                repository: repo.full_name.clone(),
                pull,
            });
        }
    }

    // Sort newest first
    all_items.sort_by(|a, b| b.pull.created_at.cmp(&a.pull.created_at));

    let total = all_items.len() as u32;
    Ok(Json(PaginatedResponse::new(
        all_items,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

/// GET /api/v1/aggregate/pulls/metrics
pub async fn aggregate_pull_metrics(
    State(state): State<AppState>,
    Query(query): Query<PullMetricsQuery>,
) -> Result<Json<ApiResponse<AggregatePullMetrics>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate metrics".into()))?;

    let repos = get_synced_repos(&state).await?;
    let state_filter = query.state.as_deref();
    let days_filter = query.days;

    let mut all_pulls: Vec<PullRequest> = Vec::new();
    let mut by_repository = Vec::new();
    let mut total_total = 0usize;
    let mut total_open = 0usize;
    let mut total_merged = 0usize;
    let mut all_merge_hours = Vec::new();

    for repo in &repos {
        let pulls = convert::pulls_from_db(pool, repo.db_id, None, None).await?;
        let filtered = filter_pulls(pulls, state_filter, days_filter);

        let metrics = PullMetricsCalculator::calculate(&filtered);

        total_total += metrics.total;
        total_open += metrics.open;
        total_merged += metrics.merged;

        if let Some(avg) = metrics.avg_time_to_merge_hours {
            for _ in 0..metrics.merged {
                all_merge_hours.push(avg);
            }
        }

        by_repository.push(RepoPullMetrics {
            repository: repo.full_name.clone(),
            total: metrics.total,
            open: metrics.open,
            merged: metrics.merged,
            avg_time_to_merge_hours: metrics.avg_time_to_merge_hours,
        });

        all_pulls.extend(filtered);
    }

    let avg_merge = if all_merge_hours.is_empty() {
        None
    } else {
        Some(all_merge_hours.iter().sum::<f64>() / all_merge_hours.len() as f64)
    };

    // Combined distribution metrics across all repos
    let combined = PullMetricsCalculator::calculate(&all_pulls);

    Ok(Json(ApiResponse::ok(AggregatePullMetrics {
        by_repository,
        totals: PullMetricsTotals {
            total: total_total,
            open: total_open,
            merged: total_merged,
            avg_time_to_merge_hours: avg_merge,
        },
        by_size: combined.by_size,
        by_author: combined.by_author,
    })))
}

/// GET /api/v1/aggregate/contributors
pub async fn aggregate_contributors(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<AggregateContributorStats>>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate metrics".into()))?;

    let repos = get_synced_repos(&state).await?;

    struct ContribAccum {
        repos: Vec<String>,
        issues_created: usize,
        prs_created: usize,
        prs_merged: usize,
    }

    let mut contributors: HashMap<String, ContribAccum> = HashMap::new();

    for repo in &repos {
        let issues = convert::issues_from_db(pool, repo.db_id, None, None).await?;
        for issue in &issues {
            let entry = contributors
                .entry(issue.author.login.clone())
                .or_insert_with(|| ContribAccum {
                    repos: vec![],
                    issues_created: 0,
                    prs_created: 0,
                    prs_merged: 0,
                });
            entry.issues_created += 1;
            if !entry.repos.contains(&repo.full_name) {
                entry.repos.push(repo.full_name.clone());
            }
        }

        let pulls = convert::pulls_from_db(pool, repo.db_id, None, None).await?;
        for pr in &pulls {
            let entry = contributors
                .entry(pr.author.login.clone())
                .or_insert_with(|| ContribAccum {
                    repos: vec![],
                    issues_created: 0,
                    prs_created: 0,
                    prs_merged: 0,
                });
            entry.prs_created += 1;
            if pr.merged {
                entry.prs_merged += 1;
            }
            if !entry.repos.contains(&repo.full_name) {
                entry.repos.push(repo.full_name.clone());
            }
        }
    }

    let mut result: Vec<AggregateContributorStats> = contributors
        .into_iter()
        .map(|(login, accum)| AggregateContributorStats {
            login,
            repositories: accum.repos,
            total_issues_created: accum.issues_created,
            total_prs_created: accum.prs_created,
            total_prs_merged: accum.prs_merged,
        })
        .collect();

    // Sort by total activity (issues + PRs)
    result.sort_by(|a, b| {
        let a_total = a.total_issues_created + a.total_prs_created;
        let b_total = b.total_issues_created + b.total_prs_created;
        b_total.cmp(&a_total)
    });

    // Limit to top 30
    result.truncate(30);

    Ok(Json(ApiResponse::ok(result)))
}

/// GET /api/v1/aggregate/velocity
pub async fn aggregate_velocity(
    State(state): State<AppState>,
    Query(query): Query<VelocityQuery>,
) -> Result<Json<ApiResponse<AggregateVelocityMetrics>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate metrics".into()))?;

    let period_str = query.period.as_deref().unwrap_or("week");
    let last = query.last.unwrap_or(12);
    let period = match period_str {
        "day" => Period::Day,
        "month" => Period::Month,
        _ => Period::Week,
    };

    let repos = get_synced_repos(&state).await?;
    let mut by_repository = Vec::new();
    let mut all_issues: Vec<Issue> = Vec::new();

    for repo in &repos {
        let issues = convert::issues_from_db(pool, repo.db_id, None, None).await?;
        let velocity = VelocityCalculator::calculate(&issues, period, last);

        by_repository.push(RepoVelocityEntry {
            repository: repo.full_name.clone(),
            avg_opened: velocity.avg_opened,
            avg_closed: velocity.avg_closed,
        });

        all_issues.extend(issues);
    }

    // Combined velocity
    let combined = VelocityCalculator::calculate(&all_issues, period, last);

    Ok(Json(ApiResponse::ok(AggregateVelocityMetrics {
        period: period_str.to_string(),
        by_repository,
        combined_avg_opened: combined.avg_opened,
        combined_avg_closed: combined.avg_closed,
        trend: format!("{:?}", combined.trend).to_lowercase(),
    })))
}
