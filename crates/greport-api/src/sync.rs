//! GitHub-to-DB sync service
//!
//! Fetches data from GitHub and upserts it into PostgreSQL.

use chrono::{DateTime, Utc};
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};
use greport_core::models::{Issue, Milestone, PullRequest, Release, Repository};
use greport_core::OctocrabClient;
use greport_db::models::{
    IssueInput, MilestoneInput, PullRequestInput, ReleaseInput, RepositoryInput,
};
use greport_db::DbPool;
use serde::Serialize;

/// Result of a sync operation.
#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub repository: String,
    pub issues_synced: usize,
    pub pulls_synced: usize,
    pub releases_synced: usize,
    pub milestones_synced: usize,
    pub synced_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Sync all data for a repository from GitHub into the database.
pub async fn sync_repository(
    pool: &DbPool,
    github: &OctocrabClient,
    owner: &str,
    repo: &str,
) -> Result<SyncResult, crate::error::ApiError> {
    let repo_id = RepoId::new(owner.to_string(), repo.to_string());

    let full_name = format!("{}/{}", owner, repo);

    // 1. Sync repository metadata (required - fail if this doesn't work)
    let repository = github.get_repository(&repo_id).await.map_err(|e| {
        tracing::error!(repo = %full_name, error = ?e, "Failed to fetch repository info");
        crate::error::ApiError::BadRequest(format!(
            "Cannot access repository '{}': {}",
            full_name, e
        ))
    })?;
    let repo_input = repo_to_input(&repository);
    greport_db::queries::upsert_repository(pool, &repo_input).await?;
    let db_repo_id = repository.id;

    let mut warnings: Vec<String> = vec![];
    let mut milestones_ok = false;
    let mut issues_ok = false;
    let mut pulls_ok = false;
    let mut releases_ok = false;

    // 2. Sync milestones (non-fatal)
    let milestones_synced = match github.list_milestones(&repo_id).await {
        Ok(milestones) => {
            let count = milestones.len();
            for ms in &milestones {
                let input = milestone_to_input(ms, db_repo_id);
                if let Err(e) = greport_db::queries::upsert_milestone(pool, &input).await {
                    tracing::warn!(repo = %full_name, milestone = %ms.title, error = ?e, "Failed to upsert milestone");
                }
            }
            milestones_ok = true;
            count
        }
        Err(e) => {
            let msg = format!("Milestones: {}", e);
            tracing::warn!(repo = %full_name, error = ?e, "Failed to sync milestones, skipping");
            warnings.push(msg);
            0
        }
    };

    // 3. Sync issues (non-fatal)
    let issues_synced = match github.list_issues(&repo_id, IssueParams::all()).await {
        Ok(issues) => {
            let count = issues.len();
            for issue in &issues {
                let input = issue_to_input(issue, db_repo_id);
                if let Err(e) = greport_db::queries::upsert_issue(pool, &input).await {
                    tracing::warn!(repo = %full_name, issue = issue.number, error = ?e, "Failed to upsert issue");
                    continue;
                }

                // Sync labels
                let labels: Vec<(i64, &str, Option<&str>)> = issue
                    .labels
                    .iter()
                    .map(|l| {
                        (
                            l.id,
                            l.name.as_str(),
                            if l.color.is_empty() {
                                None
                            } else {
                                Some(l.color.as_str())
                            },
                        )
                    })
                    .collect();
                if let Err(e) = greport_db::queries::set_issue_labels(pool, issue.id, &labels).await {
                    tracing::warn!(repo = %full_name, issue = issue.number, error = ?e, "Failed to set issue labels");
                }

                // Sync assignees
                let assignees: Vec<(i64, &str)> = issue
                    .assignees
                    .iter()
                    .map(|a| (a.id, a.login.as_str()))
                    .collect();
                if let Err(e) = greport_db::queries::set_issue_assignees(pool, issue.id, &assignees).await {
                    tracing::warn!(repo = %full_name, issue = issue.number, error = ?e, "Failed to set issue assignees");
                }
            }
            issues_ok = true;
            count
        }
        Err(e) => {
            let msg = format!("Issues: {}", e);
            tracing::warn!(repo = %full_name, error = ?e, "Failed to sync issues, skipping");
            warnings.push(msg);
            0
        }
    };

    // 4. Sync pull requests (non-fatal)
    let pulls_synced = match github.list_pulls(&repo_id, PullParams::all()).await {
        Ok(pulls) => {
            let count = pulls.len();
            for pr in &pulls {
                let input = pull_to_input(pr, db_repo_id);
                if let Err(e) = greport_db::queries::upsert_pull_request(pool, &input).await {
                    tracing::warn!(repo = %full_name, pr = pr.number, error = ?e, "Failed to upsert pull request");
                }
            }
            pulls_ok = true;
            count
        }
        Err(e) => {
            let msg = format!("Pull requests: {}", e);
            tracing::warn!(repo = %full_name, error = ?e, "Failed to sync pull requests, skipping");
            warnings.push(msg);
            0
        }
    };

    // 5. Sync releases (non-fatal)
    let releases_synced = match github.list_releases(&repo_id).await {
        Ok(releases) => {
            let count = releases.len();
            for release in &releases {
                let input = release_to_input(release, db_repo_id);
                if let Err(e) = greport_db::queries::upsert_release(pool, &input).await {
                    tracing::warn!(repo = %full_name, tag = %release.tag_name, error = ?e, "Failed to upsert release");
                }
            }
            releases_ok = true;
            count
        }
        Err(e) => {
            let msg = format!("Releases: {}", e);
            tracing::warn!(repo = %full_name, error = ?e, "Failed to sync releases, skipping");
            warnings.push(msg);
            0
        }
    };

    // 6. Update sync status (only mark synced for categories that succeeded)
    greport_db::queries::upsert_sync_status(
        pool,
        db_repo_id,
        issues_ok,
        pulls_ok,
        releases_ok,
        milestones_ok,
    )
    .await?;

    let synced_at = Utc::now();
    tracing::info!(
        repository = %format!("{}/{}", owner, repo),
        issues = issues_synced,
        pulls = pulls_synced,
        releases = releases_synced,
        milestones = milestones_synced,
        warnings = warnings.len(),
        "Sync complete"
    );

    Ok(SyncResult {
        repository: format!("{}/{}", owner, repo),
        issues_synced,
        pulls_synced,
        releases_synced,
        milestones_synced,
        synced_at,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Core model -> DB input conversions
// ---------------------------------------------------------------------------

fn repo_to_input(repo: &Repository) -> RepositoryInput {
    RepositoryInput {
        id: repo.id,
        owner: repo.owner.clone(),
        name: repo.name.clone(),
        full_name: repo.full_name.clone(),
        description: repo.description.clone(),
        private: repo.private,
        default_branch: repo.default_branch.clone(),
        created_at: repo.created_at,
        updated_at: repo.updated_at,
    }
}

fn milestone_to_input(ms: &Milestone, repo_id: i64) -> MilestoneInput {
    MilestoneInput {
        id: ms.id,
        repository_id: repo_id,
        number: ms.number as i64,
        title: ms.title.clone(),
        description: ms.description.clone(),
        state: format!("{:?}", ms.state).to_lowercase(),
        open_issues: ms.open_issues as i32,
        closed_issues: ms.closed_issues as i32,
        due_on: ms.due_on,
        created_at: ms.created_at,
        closed_at: ms.closed_at,
    }
}

fn issue_to_input(issue: &Issue, repo_id: i64) -> IssueInput {
    IssueInput {
        id: issue.id,
        repository_id: repo_id,
        number: issue.number as i64,
        title: issue.title.clone(),
        body: issue.body.clone(),
        state: match issue.state {
            greport_core::models::IssueState::Open => "open".to_string(),
            greport_core::models::IssueState::Closed => "closed".to_string(),
        },
        milestone_id: issue.milestone.as_ref().map(|m| m.id),
        author_login: issue.author.login.clone(),
        author_id: issue.author.id,
        comments_count: issue.comments_count as i32,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
        closed_by_login: issue.closed_by.as_ref().map(|u| u.login.clone()),
    }
}

fn pull_to_input(pr: &PullRequest, repo_id: i64) -> PullRequestInput {
    PullRequestInput {
        id: pr.id,
        repository_id: repo_id,
        number: pr.number as i64,
        title: pr.title.clone(),
        body: pr.body.clone(),
        state: match pr.state {
            greport_core::models::PullState::Open => "open".to_string(),
            greport_core::models::PullState::Closed => "closed".to_string(),
        },
        draft: pr.draft,
        milestone_id: pr.milestone.as_ref().map(|m| m.id),
        author_login: pr.author.login.clone(),
        author_id: pr.author.id,
        head_ref: pr.head_ref.clone(),
        base_ref: pr.base_ref.clone(),
        merged: pr.merged,
        merged_at: pr.merged_at,
        additions: pr.additions as i32,
        deletions: pr.deletions as i32,
        changed_files: pr.changed_files as i32,
        created_at: pr.created_at,
        updated_at: pr.updated_at,
        closed_at: pr.closed_at,
    }
}

fn release_to_input(release: &Release, repo_id: i64) -> ReleaseInput {
    ReleaseInput {
        id: release.id,
        repository_id: repo_id,
        tag_name: release.tag_name.clone(),
        name: release.name.clone(),
        body: release.body.clone(),
        draft: release.draft,
        prerelease: release.prerelease,
        author_login: release.author.login.clone(),
        author_id: release.author.id,
        created_at: release.created_at,
        published_at: release.published_at,
    }
}
