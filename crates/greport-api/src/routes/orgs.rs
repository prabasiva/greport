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
    pub web_url: String,
    pub repo_count: usize,
}

/// Response for the orgs list endpoint.
#[derive(Serialize)]
pub struct OrgsListResponse {
    pub orgs: Vec<OrgSummary>,
    pub default_web_url: String,
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
) -> Result<Json<ApiResponse<OrgsListResponse>>, ApiError> {
    let entries = state.registry.org_entries();

    let orgs: Vec<OrgSummary> = entries
        .iter()
        .map(|e| OrgSummary {
            name: e.name.clone(),
            web_url: state.web_url_for_owner(&e.name),
            repo_count: e.repo_count,
        })
        .collect();

    let default_web_url = state.registry.default_web_url();

    Ok(Json(ApiResponse::ok(OrgsListResponse {
        orgs,
        default_web_url,
    })))
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
