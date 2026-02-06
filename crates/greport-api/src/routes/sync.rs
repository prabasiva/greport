//! Sync route handler

use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use crate::sync::{self, SyncResult};

/// POST /api/v1/repos/{owner}/{repo}/sync
///
/// Triggers a full sync of the repository data from GitHub into PostgreSQL.
pub async fn sync_repo(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<ApiResponse<SyncResult>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database not configured".to_string()))?;

    let result = sync::sync_repository(pool, &state.github, &owner, &repo).await?;

    Ok(Json(ApiResponse::ok(result)))
}
