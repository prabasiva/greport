//! Pull request route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
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
}

pub async fn list_pulls(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListPullsQuery>,
) -> Result<Json<PaginatedResponse<PullRequest>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "pulls").await {
                let db_state = match query.state.as_deref() {
                    Some("open") => Some("open"),
                    Some("closed") => Some("closed"),
                    Some("all") | None => None,
                    _ => Some("open"),
                };
                let prs = convert::pulls_from_db(pool, repo_db_id, db_state, None).await?;
                let total = prs.len() as u32;
                return Ok(Json(PaginatedResponse::new(
                    prs,
                    query.page.unwrap_or(1),
                    query.per_page.unwrap_or(30),
                    total,
                )));
            }
        }
    }

    // Fallback: GitHub API
    let repo_id = RepoId::new(owner, repo);

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

    let prs = state.github.list_pulls(&repo_id, params).await?;
    let total = prs.len() as u32;

    Ok(Json(PaginatedResponse::new(
        prs,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(30),
        total,
    )))
}

pub async fn get_metrics(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<ApiResponse<PullMetrics>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "pulls").await {
                let prs = convert::pulls_from_db(pool, repo_db_id, None, None).await?;
                let metrics = PullMetricsCalculator::calculate(&prs);
                return Ok(Json(ApiResponse::ok(metrics)));
            }
        }
    }

    // Fallback: GitHub API
    let repo_id = RepoId::new(owner, repo);
    let prs = state.github.list_pulls(&repo_id, PullParams::all()).await?;

    let metrics = PullMetricsCalculator::calculate(&prs);

    Ok(Json(ApiResponse::ok(metrics)))
}
