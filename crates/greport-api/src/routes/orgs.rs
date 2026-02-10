//! Organization listing route handlers

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;

/// Summary of an organization (no tokens exposed).
#[derive(Serialize)]
pub struct OrgSummary {
    pub name: String,
    pub base_url: Option<String>,
    pub repo_count: usize,
}

/// Repository entry within an organization.
#[derive(Serialize)]
pub struct OrgRepoEntry {
    pub name: String,
    pub full_name: String,
}

/// GET /api/v1/orgs
///
/// List all configured organizations.
pub async fn list_orgs(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<OrgSummary>>>, ApiError> {
    let entries = state.registry.org_entries();

    let orgs: Vec<OrgSummary> = entries
        .iter()
        .map(|e| OrgSummary {
            name: e.name.clone(),
            base_url: e.base_url.clone(),
            repo_count: e.repo_count,
        })
        .collect();

    Ok(Json(ApiResponse::ok(orgs)))
}

/// GET /api/v1/orgs/{org}/repos
///
/// List configured repos for a specific organization.
pub async fn list_org_repos(
    State(state): State<AppState>,
    Path(org): Path<String>,
) -> Result<Json<ApiResponse<Vec<OrgRepoEntry>>>, ApiError> {
    let org_lower = org.to_lowercase();

    let entry = state
        .registry
        .org_entries()
        .iter()
        .find(|e| e.name.to_lowercase() == org_lower);

    match entry {
        Some(e) => {
            let repos: Vec<OrgRepoEntry> = e
                .repo_names
                .iter()
                .map(|r| OrgRepoEntry {
                    name: r.clone(),
                    full_name: format!("{}/{}", e.name, r),
                })
                .collect();

            Ok(Json(ApiResponse::ok(repos)))
        }
        None => {
            if state.registry.has_org(&org) {
                // Org exists but has no repos configured in org_entries
                Ok(Json(ApiResponse::ok(Vec::new())))
            } else {
                Err(ApiError::NotFound(format!(
                    "Organization '{}' not found",
                    org
                )))
            }
        }
    }
}
