//! Repository management route handlers

use axum::{
    extract::{Json as AxumJson, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use crate::sync;

#[derive(Serialize)]
pub struct RepoSummary {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub sync_status: Option<SyncStatusSummary>,
}

#[derive(Serialize)]
pub struct SyncStatusSummary {
    pub issues_synced: bool,
    pub pulls_synced: bool,
    pub releases_synced: bool,
    pub milestones_synced: bool,
    pub last_synced_at: Option<String>,
}

#[derive(Deserialize)]
pub struct AddRepoRequest {
    pub full_name: String,
}

/// GET /api/v1/repos - List all tracked repositories
pub async fn list_repos(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<RepoSummary>>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required".into()))?;

    let repos = greport_db::queries::list_repositories(pool).await?;

    let mut summaries = Vec::new();
    for repo in repos {
        let sync_status = match greport_db::queries::get_sync_status(pool, repo.id).await? {
            Some(status) => Some(SyncStatusSummary {
                issues_synced: status.issues_synced_at.is_some(),
                pulls_synced: status.pulls_synced_at.is_some(),
                releases_synced: status.releases_synced_at.is_some(),
                milestones_synced: status.milestones_synced_at.is_some(),
                last_synced_at: status
                    .issues_synced_at
                    .or(status.pulls_synced_at)
                    .map(|dt| dt.to_rfc3339()),
            }),
            None => None,
        };

        summaries.push(RepoSummary {
            owner: repo.owner,
            name: repo.name,
            full_name: repo.full_name,
            description: repo.description,
            sync_status,
        });
    }

    Ok(Json(ApiResponse::ok(summaries)))
}

/// POST /api/v1/repos - Add a repository and trigger initial sync
pub async fn add_repo(
    State(state): State<AppState>,
    AxumJson(body): AxumJson<AddRepoRequest>,
) -> Result<Json<ApiResponse<RepoSummary>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required".into()))?;

    let parts: Vec<&str> = body.full_name.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(ApiError::BadRequest(
            "Invalid repository format. Use owner/repo".into(),
        ));
    }
    let (owner, repo) = (parts[0], parts[1]);

    // Enforce 5-repo limit
    let existing = greport_db::queries::list_repositories(pool).await?;
    let already_tracked = existing.iter().any(|r| r.full_name == body.full_name);
    if !already_tracked && existing.len() >= 5 {
        return Err(ApiError::BadRequest(
            "Maximum 5 repositories allowed. Remove a repository before adding a new one.".into(),
        ));
    }

    // Sync the repository (this also upserts it into the DB)
    let client = state.client_for_owner(owner)?;
    let result = sync::sync_repository(pool, client.as_ref(), owner, repo).await?;

    Ok(Json(ApiResponse::ok(RepoSummary {
        owner: owner.to_string(),
        name: repo.to_string(),
        full_name: result.repository,
        description: None,
        sync_status: Some(SyncStatusSummary {
            issues_synced: true,
            pulls_synced: true,
            releases_synced: true,
            milestones_synced: true,
            last_synced_at: Some(result.synced_at.to_rfc3339()),
        }),
    })))
}

/// DELETE /api/v1/repos/{owner}/{repo} - Remove a tracked repository
pub async fn remove_repo(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required".into()))?;

    let full_name = format!("{}/{}", owner, repo);
    let repo_row = greport_db::queries::get_repository_by_name(pool, &full_name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Repository not found: {}", full_name)))?;

    greport_db::queries::delete_repository(pool, repo_row.id).await?;

    Ok(Json(ApiResponse::ok(())))
}
