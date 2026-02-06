//! Release route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

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
    let repo_id = RepoId::new(owner, repo);

    let releases = state.github.list_releases(&repo_id).await?;
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
    let repo_id = RepoId::new(owner, repo);

    // Get milestone
    let milestones = state.github.list_milestones(&repo_id).await?;
    let ms = milestones
        .iter()
        .find(|m| m.title.eq_ignore_ascii_case(&query.milestone))
        .ok_or_else(|| ApiError::NotFound(format!("Milestone not found: {}", query.milestone)))?;

    // Get closed issues for milestone
    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::closed())
        .await?;
    let milestone_issues: Vec<_> = issues
        .into_iter()
        .filter(|i| i.milestone.as_ref().map(|m| m.id) == Some(ms.id))
        .filter(|i| i.state == IssueState::Closed)
        .collect();

    // Get merged PRs
    let prs = state.github.list_pulls(&repo_id, PullParams::all()).await?;
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
    let repo_id = RepoId::new(owner, repo);

    let milestones = state.github.list_milestones(&repo_id).await?;
    let ms = milestones
        .into_iter()
        .find(|m| m.title.eq_ignore_ascii_case(&milestone))
        .ok_or_else(|| ApiError::NotFound(format!("Milestone not found: {}", milestone)))?;

    Ok(Json(ApiResponse::ok(ms)))
}
