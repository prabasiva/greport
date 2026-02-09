//! Release route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::convert;
use crate::error::ApiError;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};
use greport_core::models::{IssueState, Milestone, Release};
use greport_core::reports::{ReleaseNotes, ReleaseNotesGenerator};

#[derive(Deserialize)]
pub struct ListReleasesQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

pub async fn list_releases(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListReleasesQuery>,
) -> Result<Json<PaginatedResponse<Release>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "releases").await {
                let releases = convert::releases_from_db(pool, repo_db_id, None).await?;
                let total = releases.len() as u32;
                return Ok(Json(PaginatedResponse::new(
                    releases,
                    query.page.unwrap_or(1),
                    query.per_page.unwrap_or(10),
                    total,
                )));
            }
        }
    }

    // Fallback: GitHub API
    let repo_id = RepoId::new(owner.clone(), repo.clone());

    let client = state.client_for_owner(&owner)?;
    let releases = match client.list_releases(&repo_id).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!(
                "Failed to fetch releases from GitHub for {}/{}: {}",
                owner,
                repo,
                e
            );
            Vec::new()
        }
    };
    let total = releases.len() as u32;

    Ok(Json(PaginatedResponse::new(
        releases,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(10),
        total,
    )))
}

#[derive(Deserialize)]
pub struct ReleaseNotesQuery {
    milestone: String,
    version: Option<String>,
}

pub async fn get_notes(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleaseNotesQuery>,
) -> Result<Json<ApiResponse<ReleaseNotes>>, ApiError> {
    // DB-first: needs milestones, issues, and pulls all synced
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "milestones").await
                && convert::has_synced_data(pool, repo_db_id, "issues").await
                && convert::has_synced_data(pool, repo_db_id, "pulls").await
            {
                let milestones = convert::milestones_from_db(pool, repo_db_id).await?;
                let ms = milestones
                    .iter()
                    .find(|m| m.title.eq_ignore_ascii_case(&query.milestone))
                    .ok_or_else(|| {
                        ApiError::NotFound(format!("Milestone not found: {}", query.milestone))
                    })?;

                let issues = convert::issues_from_db(pool, repo_db_id, None, None).await?;
                let milestone_issues: Vec<_> = issues
                    .into_iter()
                    .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(ms.id))
                    .filter(|i| i.state == IssueState::Closed)
                    .collect();

                let prs = convert::pulls_from_db(pool, repo_db_id, None, None).await?;
                let merged_prs: Vec<_> = prs.into_iter().filter(|p| p.merged).collect();

                let generator = ReleaseNotesGenerator::with_defaults();
                let version = query.version.unwrap_or_else(|| query.milestone.clone());
                let notes = generator.generate(&version, &milestone_issues, &merged_prs);

                return Ok(Json(ApiResponse::ok(notes)));
            }
        }
    }

    // Fallback: GitHub API
    let client = state.client_for_owner(&owner)?;
    let repo_id = RepoId::new(owner, repo);

    // Get milestone
    let milestones = client.list_milestones(&repo_id).await?;
    let ms = milestones
        .iter()
        .find(|m| m.title.eq_ignore_ascii_case(&query.milestone))
        .ok_or_else(|| ApiError::NotFound(format!("Milestone not found: {}", query.milestone)))?;

    // Get closed issues for milestone
    let issues = client.list_issues(&repo_id, IssueParams::closed()).await?;
    let milestone_issues: Vec<_> = issues
        .into_iter()
        .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(ms.id))
        .filter(|i| i.state == IssueState::Closed)
        .collect();

    // Get merged PRs
    let prs = client.list_pulls(&repo_id, PullParams::all()).await?;
    let merged_prs: Vec<_> = prs.into_iter().filter(|p| p.merged).collect();

    // Generate notes
    let generator = ReleaseNotesGenerator::with_defaults();
    let version = query.version.unwrap_or_else(|| query.milestone.clone());
    let notes = generator.generate(&version, &milestone_issues, &merged_prs);

    Ok(Json(ApiResponse::ok(notes)))
}

pub async fn get_progress(
    State(state): State<AppState>,
    Path((owner, repo, milestone)): Path<(String, String, String)>,
) -> Result<Json<ApiResponse<Milestone>>, ApiError> {
    // DB-first
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            if convert::has_synced_data(pool, repo_db_id, "milestones").await {
                let milestones = convert::milestones_from_db(pool, repo_db_id).await?;
                let ms = milestones
                    .into_iter()
                    .find(|m| m.title.eq_ignore_ascii_case(&milestone))
                    .ok_or_else(|| {
                        ApiError::NotFound(format!("Milestone not found: {}", milestone))
                    })?;
                return Ok(Json(ApiResponse::ok(ms)));
            }
        }
    }

    // Fallback: GitHub API
    let client = state.client_for_owner(&owner)?;
    let repo_id = RepoId::new(owner, repo);

    let milestones = client.list_milestones(&repo_id).await?;
    let ms = milestones
        .into_iter()
        .find(|m| m.title.eq_ignore_ascii_case(&milestone))
        .ok_or_else(|| ApiError::NotFound(format!("Milestone not found: {}", milestone)))?;

    Ok(Json(ApiResponse::ok(ms)))
}
