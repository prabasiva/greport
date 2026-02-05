//! Issue route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::ApiError;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;
use greport_core::client::{GitHubClient, IssueParams, IssueStateFilter, RepoId};
use greport_core::metrics::{
    IssueMetrics, IssueMetricsCalculator, Period, VelocityCalculator, VelocityMetrics,
};
use greport_core::models::Issue;
use greport_core::reports::{BurndownCalculator, BurndownReport};

#[derive(Deserialize)]
pub struct ListIssuesQuery {
    state: Option<String>,
    labels: Option<String>,
    assignee: Option<String>,
    milestone: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

pub async fn list_issues(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListIssuesQuery>,
) -> Result<Json<PaginatedResponse<Issue>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);

    let issue_state = match query.state.as_deref() {
        Some("open") => IssueStateFilter::Open,
        Some("closed") => IssueStateFilter::Closed,
        Some("all") => IssueStateFilter::All,
        _ => IssueStateFilter::Open,
    };

    let params = IssueParams {
        state: issue_state,
        labels: query
            .labels
            .map(|l| l.split(',').map(String::from).collect()),
        assignee: query.assignee,
        milestone: query.milestone,
        per_page: query.per_page.unwrap_or(30).min(100) as usize,
        ..Default::default()
    };

    let issues = state.github.list_issues(&repo_id, params).await?;
    let total = issues.len() as u32;

    Ok(Json(PaginatedResponse::new(
        issues,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

pub async fn get_metrics(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<ApiResponse<IssueMetrics>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);
    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::all())
        .await?;

    let calculator = IssueMetricsCalculator::new(30);
    let metrics = calculator.calculate(&issues);

    Ok(Json(ApiResponse::ok(metrics)))
}

#[derive(Deserialize)]
pub struct VelocityQuery {
    period: Option<String>,
    last: Option<usize>,
}

pub async fn get_velocity(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<VelocityQuery>,
) -> Result<Json<ApiResponse<VelocityMetrics>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);
    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::all())
        .await?;

    let period = match query.period.as_deref() {
        Some("day") => Period::Day,
        Some("month") => Period::Month,
        _ => Period::Week,
    };

    let velocity = VelocityCalculator::calculate(&issues, period, query.last.unwrap_or(12));

    Ok(Json(ApiResponse::ok(velocity)))
}

#[derive(Deserialize)]
pub struct BurndownQuery {
    milestone: String,
}

pub async fn get_burndown(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<BurndownQuery>,
) -> Result<Json<ApiResponse<BurndownReport>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);

    let milestones = state.github.list_milestones(&repo_id).await?;
    let ms = milestones
        .iter()
        .find(|m| m.title.eq_ignore_ascii_case(&query.milestone))
        .ok_or_else(|| ApiError::NotFound(format!("Milestone not found: {}", query.milestone)))?;

    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::all())
        .await?;
    let burndown = BurndownCalculator::calculate(&issues, ms);

    Ok(Json(ApiResponse::ok(burndown)))
}

#[derive(Deserialize)]
pub struct StaleQuery {
    days: Option<i64>,
}

pub async fn get_stale(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<StaleQuery>,
) -> Result<Json<ApiResponse<Vec<Issue>>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);
    let days = query.days.unwrap_or(30);

    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::open())
        .await?;
    let stale: Vec<_> = issues.into_iter().filter(|i| i.is_stale(days)).collect();

    Ok(Json(ApiResponse::ok(stale)))
}
