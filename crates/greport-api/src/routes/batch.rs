//! Batch sync route handlers

use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;

use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use crate::sync;

/// Result of syncing a single repository within a batch
#[derive(Serialize)]
pub struct RepoSyncResult {
    pub repository: String,
    pub success: bool,
    pub issues_synced: Option<usize>,
    pub pulls_synced: Option<usize>,
    pub releases_synced: Option<usize>,
    pub milestones_synced: Option<usize>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Result of a batch sync operation
#[derive(Serialize)]
pub struct BatchSyncResult {
    pub results: Vec<RepoSyncResult>,
    pub total_repos: usize,
    pub successful: usize,
    pub failed: usize,
    pub synced_at: String,
}

/// POST /api/v1/sync - Batch sync all tracked repositories
pub async fn batch_sync(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<BatchSyncResult>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for batch sync".into()))?;

    // Get tracked repos from DB
    let tracked = greport_db::queries::list_tracked_repos(pool).await?;

    if tracked.is_empty() {
        return Ok(Json(ApiResponse::ok(BatchSyncResult {
            results: vec![],
            total_repos: 0,
            successful: 0,
            failed: 0,
            synced_at: Utc::now().to_rfc3339(),
        })));
    }

    let mut results = Vec::new();
    let mut successful = 0usize;
    let mut failed = 0usize;

    // Sync repos sequentially to respect GitHub API rate limits
    for tracked_repo in &tracked {
        let parts: Vec<&str> = tracked_repo.full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            results.push(RepoSyncResult {
                repository: tracked_repo.full_name.clone(),
                success: false,
                issues_synced: None,
                pulls_synced: None,
                releases_synced: None,
                milestones_synced: None,
                error: Some("Invalid repository format".into()),
                warnings: vec![],
            });
            failed += 1;
            continue;
        }

        let (owner, repo) = (parts[0], parts[1]);

        let client = match state.client_for_owner(owner) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    repo = %tracked_repo.full_name,
                    error = %e,
                    "No GitHub client configured for org"
                );
                results.push(RepoSyncResult {
                    repository: tracked_repo.full_name.clone(),
                    success: false,
                    issues_synced: None,
                    pulls_synced: None,
                    releases_synced: None,
                    milestones_synced: None,
                    error: Some(format!("{e}")),
                    warnings: vec![],
                });
                failed += 1;
                continue;
            }
        };

        match sync::sync_repository(pool, client.as_ref(), owner, repo).await {
            Ok(result) => {
                results.push(RepoSyncResult {
                    repository: result.repository,
                    success: true,
                    issues_synced: Some(result.issues_synced),
                    pulls_synced: Some(result.pulls_synced),
                    releases_synced: Some(result.releases_synced),
                    milestones_synced: Some(result.milestones_synced),
                    error: None,
                    warnings: result.warnings,
                });
                successful += 1;
            }
            Err(e) => {
                tracing::warn!(
                    repo = %tracked_repo.full_name,
                    error = ?e,
                    "Batch sync failed for repo"
                );
                results.push(RepoSyncResult {
                    repository: tracked_repo.full_name.clone(),
                    success: false,
                    issues_synced: None,
                    pulls_synced: None,
                    releases_synced: None,
                    milestones_synced: None,
                    error: Some(format!("{e}")),
                    warnings: vec![],
                });
                failed += 1;
            }
        }
    }

    Ok(Json(ApiResponse::ok(BatchSyncResult {
        total_repos: results.len(),
        results,
        successful,
        failed,
        synced_at: Utc::now().to_rfc3339(),
    })))
}
