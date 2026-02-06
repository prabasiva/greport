//! Contributor route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};

#[derive(Deserialize)]
pub struct ContributorsQuery {
    sort_by: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
pub struct ContributorStats {
    pub login: String,
    pub issues_created: usize,
    pub prs_created: usize,
    pub prs_merged: usize,
}

pub async fn list_contributors(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ContributorsQuery>,
) -> Result<Json<ApiResponse<Vec<ContributorStats>>>, ApiError> {
    let repo_id = RepoId::new(owner, repo);

    let issues = state
        .github
        .list_issues(&repo_id, IssueParams::all())
        .await?;
    let prs = state.github.list_pulls(&repo_id, PullParams::all()).await?;

    let mut contributors: HashMap<String, ContributorStats> = HashMap::new();

    for issue in &issues {
        let entry = contributors
            .entry(issue.author.login.clone())
            .or_insert_with(|| ContributorStats {
                login: issue.author.login.clone(),
                issues_created: 0,
                prs_created: 0,
                prs_merged: 0,
            });
        entry.issues_created += 1;
    }

    for pr in &prs {
        let entry = contributors
            .entry(pr.author.login.clone())
            .or_insert_with(|| ContributorStats {
                login: pr.author.login.clone(),
                issues_created: 0,
                prs_created: 0,
                prs_merged: 0,
            });
        entry.prs_created += 1;
        if pr.merged {
            entry.prs_merged += 1;
        }
    }

    let mut sorted: Vec<_> = contributors.into_values().collect();

    match query.sort_by.as_deref() {
        Some("prs") => sorted.sort_by(|a, b| b.prs_created.cmp(&a.prs_created)),
        _ => sorted.sort_by(|a, b| b.issues_created.cmp(&a.issues_created)),
    }

    let limit = query.limit.unwrap_or(20);
    sorted.truncate(limit);

    Ok(Json(ApiResponse::ok(sorted)))
}
