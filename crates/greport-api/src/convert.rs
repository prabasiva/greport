//! DB-to-model conversion layer
//!
//! Reads from PostgreSQL and converts flat DB rows into rich core domain models.

use greport_core::models::{
    Issue, IssueState, Label, Milestone, MilestoneState, PullRequest, PullState, Release, User,
};
use greport_db::models::{IssueRow, MilestoneRow, PullRequestRow, ReleaseRow};
use greport_db::DbPool;

/// Look up the internal DB id for a repository by owner/repo.
pub async fn get_repo_db_id(pool: &DbPool, owner: &str, repo: &str) -> Option<i64> {
    let full_name = format!("{}/{}", owner, repo);
    greport_db::queries::get_repository_by_name(pool, &full_name)
        .await
        .ok()
        .flatten()
        .map(|r| r.id)
}

/// Check whether data of the given type has been synced for this repository.
pub async fn has_synced_data(pool: &DbPool, repo_db_id: i64, data_type: &str) -> bool {
    let status = match greport_db::queries::get_sync_status(pool, repo_db_id).await {
        Ok(Some(s)) => s,
        _ => return false,
    };

    match data_type {
        "issues" => status.issues_synced_at.is_some(),
        "pulls" => status.pulls_synced_at.is_some(),
        "releases" => status.releases_synced_at.is_some(),
        "milestones" => status.milestones_synced_at.is_some(),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Helper: synthesize a User from login + id stored in a DB row
// ---------------------------------------------------------------------------
fn user_from_db(login: &str, id: i64) -> User {
    User {
        id,
        login: login.to_string(),
        avatar_url: format!("https://avatars.githubusercontent.com/u/{}", id),
        html_url: format!("https://github.com/{}", login),
    }
}

// ---------------------------------------------------------------------------
// Issues
// ---------------------------------------------------------------------------

/// Fetch issues from DB and convert to core Issue models.
pub async fn issues_from_db(
    pool: &DbPool,
    repo_db_id: i64,
    state: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<Issue>, sqlx::Error> {
    let rows = greport_db::queries::list_issues(pool, repo_db_id, state, None, limit).await?;

    let mut issues = Vec::with_capacity(rows.len());
    for row in rows {
        let issue = issue_row_to_model(pool, row).await?;
        issues.push(issue);
    }
    Ok(issues)
}

async fn issue_row_to_model(pool: &DbPool, row: IssueRow) -> Result<Issue, sqlx::Error> {
    // Fetch labels
    let label_rows = greport_db::queries::get_issue_labels(pool, row.id).await?;
    let labels: Vec<Label> = label_rows
        .into_iter()
        .map(|l| Label {
            id: l.label_id,
            name: l.label_name,
            color: l.label_color.unwrap_or_default(),
            description: None,
        })
        .collect();

    // Fetch assignees
    let assignee_rows = greport_db::queries::get_issue_assignees(pool, row.id).await?;
    let assignees: Vec<User> = assignee_rows
        .into_iter()
        .map(|a| user_from_db(&a.user_login, a.user_id))
        .collect();

    // Fetch milestone if present
    let milestone = match row.milestone_id {
        Some(ms_id) => greport_db::queries::get_milestone(pool, ms_id)
            .await?
            .map(milestone_row_to_model),
        None => None,
    };

    let state = match row.state.as_str() {
        "closed" => IssueState::Closed,
        _ => IssueState::Open,
    };

    let closed_by = row
        .closed_by_login
        .as_ref()
        .map(|login| user_from_db(login, 0));

    Ok(Issue {
        id: row.id,
        number: row.number as u64,
        title: row.title,
        body: row.body,
        state,
        labels,
        assignees,
        milestone,
        author: user_from_db(&row.author_login, row.author_id),
        comments_count: row.comments_count as u32,
        created_at: row.created_at,
        updated_at: row.updated_at,
        closed_at: row.closed_at,
        closed_by,
    })
}

// ---------------------------------------------------------------------------
// Pull Requests
// ---------------------------------------------------------------------------

/// Fetch pull requests from DB and convert to core PullRequest models.
pub async fn pulls_from_db(
    pool: &DbPool,
    repo_db_id: i64,
    state: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<PullRequest>, sqlx::Error> {
    let rows = greport_db::queries::list_pull_requests(pool, repo_db_id, state, limit).await?;

    let pulls: Vec<PullRequest> = rows.into_iter().map(pull_row_to_model).collect();
    Ok(pulls)
}

fn pull_row_to_model(row: PullRequestRow) -> PullRequest {
    let state = match row.state.as_str() {
        "closed" => PullState::Closed,
        _ => PullState::Open,
    };

    // Fetch milestone if needed (skipped for PRs – no PR-labels table in DB)
    PullRequest {
        id: row.id,
        number: row.number as u64,
        title: row.title,
        body: row.body,
        state,
        draft: row.draft,
        author: user_from_db(&row.author_login, row.author_id),
        labels: vec![],
        milestone: None,
        head_ref: row.head_ref,
        base_ref: row.base_ref,
        merged: row.merged,
        merged_at: row.merged_at,
        additions: row.additions as u32,
        deletions: row.deletions as u32,
        changed_files: row.changed_files as u32,
        created_at: row.created_at,
        updated_at: row.updated_at,
        closed_at: row.closed_at,
    }
}

// ---------------------------------------------------------------------------
// Releases
// ---------------------------------------------------------------------------

/// Fetch releases from DB and convert to core Release models.
pub async fn releases_from_db(
    pool: &DbPool,
    repo_db_id: i64,
    limit: Option<i64>,
) -> Result<Vec<Release>, sqlx::Error> {
    let rows = greport_db::queries::list_releases(pool, repo_db_id, limit).await?;

    let releases: Vec<Release> = rows.into_iter().map(release_row_to_model).collect();
    Ok(releases)
}

fn release_row_to_model(row: ReleaseRow) -> Release {
    Release {
        id: row.id,
        tag_name: row.tag_name,
        name: row.name,
        body: row.body,
        draft: row.draft,
        prerelease: row.prerelease,
        author: user_from_db(&row.author_login, row.author_id),
        created_at: row.created_at,
        published_at: row.published_at,
    }
}

// ---------------------------------------------------------------------------
// Milestones
// ---------------------------------------------------------------------------

/// Fetch milestones from DB and convert to core Milestone models.
pub async fn milestones_from_db(
    pool: &DbPool,
    repo_db_id: i64,
) -> Result<Vec<Milestone>, sqlx::Error> {
    let rows = greport_db::queries::list_milestones(pool, repo_db_id, None).await?;

    let milestones: Vec<Milestone> = rows.into_iter().map(milestone_row_to_model).collect();
    Ok(milestones)
}

fn milestone_row_to_model(row: MilestoneRow) -> Milestone {
    let state = match row.state.as_str() {
        "closed" => MilestoneState::Closed,
        _ => MilestoneState::Open,
    };

    Milestone {
        id: row.id,
        number: row.number as u64,
        title: row.title,
        description: row.description,
        state,
        open_issues: row.open_issues as u32,
        closed_issues: row.closed_issues as u32,
        due_on: row.due_on,
        created_at: row.created_at,
        closed_at: row.closed_at,
    }
}
