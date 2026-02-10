//! Pull request route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;

use crate::convert;
use crate::error::ApiError;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;
use greport_core::client::{GitHubClient, PullParams, PullStateFilter, RepoId};
use greport_core::metrics::{PullMetrics, PullMetricsCalculator};
use greport_core::models::PullRequest;

#[derive(Deserialize)]
pub struct ListPullsQuery {
    state: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
    days: Option<i64>,
}

pub async fn list_pulls(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListPullsQuery>,
) -> Result<Json<PaginatedResponse<PullRequest>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            let db_state = match query.state.as_deref() {
                Some("open") => Some("open"),
                Some("closed") => Some("closed"),
                Some("all") | None => None,
                _ => Some("open"),
            };
            let mut prs = convert::pulls_from_db(pool, repo_db_id, db_state, None).await?;
            if let Some(d) = query.days {
                let cutoff = Utc::now() - chrono::Duration::days(d);
                prs.retain(|p| p.created_at >= cutoff);
            }
            let total = prs.len() as u32;
            return Ok(Json(PaginatedResponse::new(
                prs,
                query.page.unwrap_or(1),
                query.per_page.unwrap_or(30),
                total,
            )));
        }
    }

    // Fallback: GitHub API
    let repo_id = RepoId::new(owner.clone(), repo.clone());

    let pr_state = match query.state.as_deref() {
        Some("open") => PullStateFilter::Open,
        Some("closed") => PullStateFilter::Closed,
        Some("all") => PullStateFilter::All,
        _ => PullStateFilter::Open,
    };

    let params = PullParams {
        state: pr_state,
        per_page: query.per_page.unwrap_or(30).min(100) as usize,
        ..Default::default()
    };

    let client = state.client_for_owner(&owner)?;
    let prs = match client.list_pulls(&repo_id, params).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!(
                "Failed to fetch pull requests from GitHub for {}/{}: {}",
                owner,
                repo,
                e
            );
            Vec::new()
        }
    };
    let total = prs.len() as u32;

    Ok(Json(PaginatedResponse::new(
        prs,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

#[derive(Deserialize)]
pub struct MetricsQuery {
    state: Option<String>,
    days: Option<i64>,
}

pub async fn get_metrics(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<ApiResponse<PullMetrics>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "pulls").await {
                let prs = convert::pulls_from_db(pool, repo_db_id, None, None).await?;
                let filtered = filter_pulls_by(prs, query.state.as_deref(), query.days);
                let metrics = PullMetricsCalculator::calculate(&filtered);
                return Ok(Json(ApiResponse::ok(metrics)));
            }
        }
    }

    // Fallback: GitHub API
    let repo_id = RepoId::new(owner.clone(), repo.clone());
    let client = state.client_for_owner(&owner)?;
    let prs = match client.list_pulls(&repo_id, PullParams::all()).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!(
                "Failed to fetch pull metrics from GitHub for {}/{}: {}",
                owner,
                repo,
                e
            );
            Vec::new()
        }
    };

    let filtered = filter_pulls_by(prs, query.state.as_deref(), query.days);
    let metrics = PullMetricsCalculator::calculate(&filtered);

    Ok(Json(ApiResponse::ok(metrics)))
}

fn filter_pulls_by(
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
