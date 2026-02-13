//! Database queries for all tables

use crate::models::*;
use crate::DbPool;
use chrono::Utc;
use uuid::Uuid;

// =============================================================================
// Repository queries
// =============================================================================

/// Get repository by ID
pub async fn get_repository(pool: &DbPool, id: i64) -> sqlx::Result<Option<RepositoryRow>> {
    sqlx::query_as::<_, RepositoryRow>("SELECT * FROM repositories WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get repository by full name (owner/repo)
pub async fn get_repository_by_name(
    pool: &DbPool,
    full_name: &str,
) -> sqlx::Result<Option<RepositoryRow>> {
    sqlx::query_as::<_, RepositoryRow>("SELECT * FROM repositories WHERE full_name = $1")
        .bind(full_name)
        .fetch_optional(pool)
        .await
}

/// List all repositories
pub async fn list_repositories(pool: &DbPool) -> sqlx::Result<Vec<RepositoryRow>> {
    sqlx::query_as::<_, RepositoryRow>("SELECT * FROM repositories ORDER BY full_name")
        .fetch_all(pool)
        .await
}

/// List tracked repositories (alias for list_repositories)
pub async fn list_tracked_repos(pool: &DbPool) -> sqlx::Result<Vec<RepositoryRow>> {
    list_repositories(pool).await
}

/// Upsert repository
pub async fn upsert_repository(pool: &DbPool, input: &RepositoryInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO repositories (id, owner, name, full_name, description, private,
                                  default_branch, org_name, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (id) DO UPDATE SET
            owner = EXCLUDED.owner,
            name = EXCLUDED.name,
            full_name = EXCLUDED.full_name,
            description = EXCLUDED.description,
            private = EXCLUDED.private,
            default_branch = EXCLUDED.default_branch,
            org_name = COALESCE(EXCLUDED.org_name, repositories.org_name),
            updated_at = EXCLUDED.updated_at,
            synced_at = NOW()
        "#,
    )
    .bind(input.id)
    .bind(&input.owner)
    .bind(&input.name)
    .bind(&input.full_name)
    .bind(&input.description)
    .bind(input.private)
    .bind(&input.default_branch)
    .bind(&input.org_name)
    .bind(input.created_at)
    .bind(input.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete repository (cascades to related data)
pub async fn delete_repository(pool: &DbPool, id: i64) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// =============================================================================
// Milestone queries
// =============================================================================

/// Get milestone by ID
pub async fn get_milestone(pool: &DbPool, id: i64) -> sqlx::Result<Option<MilestoneRow>> {
    sqlx::query_as::<_, MilestoneRow>("SELECT * FROM milestones WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// List milestones for repository
pub async fn list_milestones(
    pool: &DbPool,
    repository_id: i64,
    state: Option<&str>,
) -> sqlx::Result<Vec<MilestoneRow>> {
    match state {
        Some(s) => {
            sqlx::query_as::<_, MilestoneRow>(
                "SELECT * FROM milestones WHERE repository_id = $1 AND state = $2 ORDER BY number",
            )
            .bind(repository_id)
            .bind(s)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, MilestoneRow>(
                "SELECT * FROM milestones WHERE repository_id = $1 ORDER BY number",
            )
            .bind(repository_id)
            .fetch_all(pool)
            .await
        }
    }
}

/// Upsert milestone
pub async fn upsert_milestone(pool: &DbPool, input: &MilestoneInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO milestones (id, repository_id, number, title, description, state,
                                open_issues, closed_issues, due_on, created_at, closed_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            state = EXCLUDED.state,
            open_issues = EXCLUDED.open_issues,
            closed_issues = EXCLUDED.closed_issues,
            due_on = EXCLUDED.due_on,
            closed_at = EXCLUDED.closed_at,
            synced_at = NOW()
        "#,
    )
    .bind(input.id)
    .bind(input.repository_id)
    .bind(input.number)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.state)
    .bind(input.open_issues)
    .bind(input.closed_issues)
    .bind(input.due_on)
    .bind(input.created_at)
    .bind(input.closed_at)
    .execute(pool)
    .await?;

    Ok(())
}

// =============================================================================
// Issue queries
// =============================================================================

/// Get issue by ID
pub async fn get_issue(pool: &DbPool, id: i64) -> sqlx::Result<Option<IssueRow>> {
    sqlx::query_as::<_, IssueRow>("SELECT * FROM issues WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get issue by repository and number
pub async fn get_issue_by_number(
    pool: &DbPool,
    repository_id: i64,
    number: i64,
) -> sqlx::Result<Option<IssueRow>> {
    sqlx::query_as::<_, IssueRow>("SELECT * FROM issues WHERE repository_id = $1 AND number = $2")
        .bind(repository_id)
        .bind(number)
        .fetch_optional(pool)
        .await
}

/// List issues for repository with optional filters
pub async fn list_issues(
    pool: &DbPool,
    repository_id: i64,
    state: Option<&str>,
    milestone_id: Option<i64>,
    limit: Option<i64>,
) -> sqlx::Result<Vec<IssueRow>> {
    let mut query = String::from("SELECT * FROM issues WHERE repository_id = $1");
    let mut param_count = 1;

    if state.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND state = ${}", param_count));
    }

    if milestone_id.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND milestone_id = ${}", param_count));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    let mut q = sqlx::query_as::<_, IssueRow>(&query).bind(repository_id);

    if let Some(s) = state {
        q = q.bind(s);
    }

    if let Some(m) = milestone_id {
        q = q.bind(m);
    }

    q.fetch_all(pool).await
}

/// Upsert issue
pub async fn upsert_issue(pool: &DbPool, input: &IssueInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO issues (id, repository_id, number, title, body, state, milestone_id,
                           author_login, author_id, comments_count, created_at, updated_at,
                           closed_at, closed_by_login, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW())
        ON CONFLICT (repository_id, number) DO UPDATE SET
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            milestone_id = EXCLUDED.milestone_id,
            comments_count = EXCLUDED.comments_count,
            updated_at = EXCLUDED.updated_at,
            closed_at = EXCLUDED.closed_at,
            closed_by_login = EXCLUDED.closed_by_login,
            synced_at = NOW()
        "#,
    )
    .bind(input.id)
    .bind(input.repository_id)
    .bind(input.number)
    .bind(&input.title)
    .bind(&input.body)
    .bind(&input.state)
    .bind(input.milestone_id)
    .bind(&input.author_login)
    .bind(input.author_id)
    .bind(input.comments_count)
    .bind(input.created_at)
    .bind(input.updated_at)
    .bind(input.closed_at)
    .bind(&input.closed_by_login)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get issue labels
pub async fn get_issue_labels(pool: &DbPool, issue_id: i64) -> sqlx::Result<Vec<IssueLabelRow>> {
    sqlx::query_as::<_, IssueLabelRow>("SELECT * FROM issue_labels WHERE issue_id = $1")
        .bind(issue_id)
        .fetch_all(pool)
        .await
}

/// Set issue labels (replace all)
pub async fn set_issue_labels(
    pool: &DbPool,
    issue_id: i64,
    labels: &[(i64, &str, Option<&str>)],
) -> sqlx::Result<()> {
    // Delete existing labels
    sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
        .bind(issue_id)
        .execute(pool)
        .await?;

    // Insert new labels
    for (label_id, name, color) in labels {
        sqlx::query(
            "INSERT INTO issue_labels (issue_id, label_id, label_name, label_color) VALUES ($1, $2, $3, $4)",
        )
        .bind(issue_id)
        .bind(label_id)
        .bind(name)
        .bind(color)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Get issue assignees
pub async fn get_issue_assignees(
    pool: &DbPool,
    issue_id: i64,
) -> sqlx::Result<Vec<IssueAssigneeRow>> {
    sqlx::query_as::<_, IssueAssigneeRow>("SELECT * FROM issue_assignees WHERE issue_id = $1")
        .bind(issue_id)
        .fetch_all(pool)
        .await
}

/// Set issue assignees (replace all)
pub async fn set_issue_assignees(
    pool: &DbPool,
    issue_id: i64,
    assignees: &[(i64, &str)],
) -> sqlx::Result<()> {
    // Delete existing assignees
    sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1")
        .bind(issue_id)
        .execute(pool)
        .await?;

    // Insert new assignees
    for (user_id, login) in assignees {
        sqlx::query(
            "INSERT INTO issue_assignees (issue_id, user_id, user_login) VALUES ($1, $2, $3)",
        )
        .bind(issue_id)
        .bind(user_id)
        .bind(login)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Count issues by state
pub async fn count_issues_by_state(
    pool: &DbPool,
    repository_id: i64,
) -> sqlx::Result<Vec<(String, i64)>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT state, COUNT(*) as count FROM issues WHERE repository_id = $1 GROUP BY state",
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// =============================================================================
// Pull request queries
// =============================================================================

/// Get pull request by ID
pub async fn get_pull_request(pool: &DbPool, id: i64) -> sqlx::Result<Option<PullRequestRow>> {
    sqlx::query_as::<_, PullRequestRow>("SELECT * FROM pull_requests WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get pull request by repository and number
pub async fn get_pull_request_by_number(
    pool: &DbPool,
    repository_id: i64,
    number: i64,
) -> sqlx::Result<Option<PullRequestRow>> {
    sqlx::query_as::<_, PullRequestRow>(
        "SELECT * FROM pull_requests WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await
}

/// List pull requests for repository
pub async fn list_pull_requests(
    pool: &DbPool,
    repository_id: i64,
    state: Option<&str>,
    limit: Option<i64>,
) -> sqlx::Result<Vec<PullRequestRow>> {
    let mut query = String::from("SELECT * FROM pull_requests WHERE repository_id = $1");

    if state.is_some() {
        query.push_str(" AND state = $2");
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    if let Some(s) = state {
        sqlx::query_as::<_, PullRequestRow>(&query)
            .bind(repository_id)
            .bind(s)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, PullRequestRow>(&query)
            .bind(repository_id)
            .fetch_all(pool)
            .await
    }
}

/// Upsert pull request
pub async fn upsert_pull_request(pool: &DbPool, input: &PullRequestInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO pull_requests (id, repository_id, number, title, body, state, draft,
                                   milestone_id, author_login, author_id, head_ref, base_ref,
                                   merged, merged_at, additions, deletions, changed_files,
                                   created_at, updated_at, closed_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, NOW())
        ON CONFLICT (repository_id, number) DO UPDATE SET
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            draft = EXCLUDED.draft,
            milestone_id = EXCLUDED.milestone_id,
            head_ref = EXCLUDED.head_ref,
            base_ref = EXCLUDED.base_ref,
            merged = EXCLUDED.merged,
            merged_at = EXCLUDED.merged_at,
            additions = EXCLUDED.additions,
            deletions = EXCLUDED.deletions,
            changed_files = EXCLUDED.changed_files,
            updated_at = EXCLUDED.updated_at,
            closed_at = EXCLUDED.closed_at,
            synced_at = NOW()
        "#,
    )
    .bind(input.id)
    .bind(input.repository_id)
    .bind(input.number)
    .bind(&input.title)
    .bind(&input.body)
    .bind(&input.state)
    .bind(input.draft)
    .bind(input.milestone_id)
    .bind(&input.author_login)
    .bind(input.author_id)
    .bind(&input.head_ref)
    .bind(&input.base_ref)
    .bind(input.merged)
    .bind(input.merged_at)
    .bind(input.additions)
    .bind(input.deletions)
    .bind(input.changed_files)
    .bind(input.created_at)
    .bind(input.updated_at)
    .bind(input.closed_at)
    .execute(pool)
    .await?;

    Ok(())
}

// =============================================================================
// Release queries
// =============================================================================

/// Get release by ID
pub async fn get_release(pool: &DbPool, id: i64) -> sqlx::Result<Option<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>("SELECT * FROM releases WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get release by repository and tag
pub async fn get_release_by_tag(
    pool: &DbPool,
    repository_id: i64,
    tag_name: &str,
) -> sqlx::Result<Option<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>(
        "SELECT * FROM releases WHERE repository_id = $1 AND tag_name = $2",
    )
    .bind(repository_id)
    .bind(tag_name)
    .fetch_optional(pool)
    .await
}

/// List releases for repository
pub async fn list_releases(
    pool: &DbPool,
    repository_id: i64,
    limit: Option<i64>,
) -> sqlx::Result<Vec<ReleaseRow>> {
    let query = match limit {
        Some(l) => format!(
            "SELECT * FROM releases WHERE repository_id = $1 ORDER BY created_at DESC LIMIT {}",
            l
        ),
        None => {
            "SELECT * FROM releases WHERE repository_id = $1 ORDER BY created_at DESC".to_string()
        }
    };

    sqlx::query_as::<_, ReleaseRow>(&query)
        .bind(repository_id)
        .fetch_all(pool)
        .await
}

/// Upsert release
pub async fn upsert_release(pool: &DbPool, input: &ReleaseInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO releases (id, repository_id, tag_name, name, body, draft, prerelease,
                              author_login, author_id, created_at, published_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
        ON CONFLICT (repository_id, tag_name) DO UPDATE SET
            name = EXCLUDED.name,
            body = EXCLUDED.body,
            draft = EXCLUDED.draft,
            prerelease = EXCLUDED.prerelease,
            published_at = EXCLUDED.published_at,
            synced_at = NOW()
        "#,
    )
    .bind(input.id)
    .bind(input.repository_id)
    .bind(&input.tag_name)
    .bind(&input.name)
    .bind(&input.body)
    .bind(input.draft)
    .bind(input.prerelease)
    .bind(&input.author_login)
    .bind(input.author_id)
    .bind(input.created_at)
    .bind(input.published_at)
    .execute(pool)
    .await?;

    Ok(())
}

// =============================================================================
// API key queries
// =============================================================================

/// Get API key by hash
pub async fn get_api_key_by_hash(pool: &DbPool, key_hash: &str) -> sqlx::Result<Option<ApiKeyRow>> {
    sqlx::query_as::<_, ApiKeyRow>(
        r#"
        SELECT * FROM api_keys
        WHERE key_hash = $1
        AND revoked = FALSE
        AND (expires_at IS NULL OR expires_at > NOW())
        "#,
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await
}

/// List API keys for owner
pub async fn list_api_keys(pool: &DbPool, owner: &str) -> sqlx::Result<Vec<ApiKeyRow>> {
    sqlx::query_as::<_, ApiKeyRow>(
        "SELECT * FROM api_keys WHERE owner = $1 AND revoked = FALSE ORDER BY created_at DESC",
    )
    .bind(owner)
    .fetch_all(pool)
    .await
}

/// Create API key
pub async fn create_api_key(pool: &DbPool, input: &ApiKeyInput) -> sqlx::Result<ApiKeyRow> {
    sqlx::query_as::<_, ApiKeyRow>(
        r#"
        INSERT INTO api_keys (name, key_hash, owner, scopes, rate_limit, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&input.name)
    .bind(&input.key_hash)
    .bind(&input.owner)
    .bind(&input.scopes)
    .bind(input.rate_limit)
    .bind(input.expires_at)
    .fetch_one(pool)
    .await
}

/// Update API key last used timestamp
pub async fn update_api_key_last_used(pool: &DbPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Revoke API key
pub async fn revoke_api_key(pool: &DbPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query("UPDATE api_keys SET revoked = TRUE WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// =============================================================================
// Sync status queries
// =============================================================================

/// Get sync status for repository
pub async fn get_sync_status(
    pool: &DbPool,
    repository_id: i64,
) -> sqlx::Result<Option<SyncStatusRow>> {
    sqlx::query_as::<_, SyncStatusRow>("SELECT * FROM sync_status WHERE repository_id = $1")
        .bind(repository_id)
        .fetch_optional(pool)
        .await
}

/// Upsert sync status
pub async fn upsert_sync_status(
    pool: &DbPool,
    repository_id: i64,
    issues_synced: bool,
    pulls_synced: bool,
    releases_synced: bool,
    milestones_synced: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sync_status (repository_id, issues_synced_at, pulls_synced_at,
                                 releases_synced_at, milestones_synced_at)
        VALUES ($1,
                CASE WHEN $2 THEN NOW() ELSE NULL END,
                CASE WHEN $3 THEN NOW() ELSE NULL END,
                CASE WHEN $4 THEN NOW() ELSE NULL END,
                CASE WHEN $5 THEN NOW() ELSE NULL END)
        ON CONFLICT (repository_id) DO UPDATE SET
            issues_synced_at = CASE WHEN $2 THEN NOW() ELSE sync_status.issues_synced_at END,
            pulls_synced_at = CASE WHEN $3 THEN NOW() ELSE sync_status.pulls_synced_at END,
            releases_synced_at = CASE WHEN $4 THEN NOW() ELSE sync_status.releases_synced_at END,
            milestones_synced_at = CASE WHEN $5 THEN NOW() ELSE sync_status.milestones_synced_at END
        "#,
    )
    .bind(repository_id)
    .bind(issues_synced)
    .bind(pulls_synced)
    .bind(releases_synced)
    .bind(milestones_synced)
    .execute(pool)
    .await?;

    Ok(())
}

/// Record sync error
pub async fn record_sync_error(pool: &DbPool, repository_id: i64, error: &str) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sync_status (repository_id, last_error, last_error_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (repository_id) DO UPDATE SET
            last_error = $2,
            last_error_at = NOW()
        "#,
    )
    .bind(repository_id)
    .bind(error)
    .execute(pool)
    .await?;

    Ok(())
}

// =============================================================================
// Cache metadata queries
// =============================================================================

/// Get cache entry
pub async fn get_cache_entry(pool: &DbPool, key: &str) -> sqlx::Result<Option<CacheMetadataRow>> {
    let row = sqlx::query_as::<_, CacheMetadataRow>(
        "SELECT * FROM cache_metadata WHERE key = $1 AND expires_at > NOW()",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    // Increment hit count if found
    if row.is_some() {
        sqlx::query("UPDATE cache_metadata SET hit_count = hit_count + 1 WHERE key = $1")
            .bind(key)
            .execute(pool)
            .await?;
    }

    Ok(row)
}

/// Set cache entry
pub async fn set_cache_entry(
    pool: &DbPool,
    key: &str,
    data_type: &str,
    ttl_seconds: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO cache_metadata (key, data_type, expires_at)
        VALUES ($1, $2, NOW() + make_interval(secs => $3))
        ON CONFLICT (key) DO UPDATE SET
            data_type = EXCLUDED.data_type,
            expires_at = NOW() + make_interval(secs => $3),
            hit_count = 0,
            created_at = NOW()
        "#,
    )
    .bind(key)
    .bind(data_type)
    .bind(ttl_seconds as f64)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired cache entries
pub async fn cleanup_expired_cache(pool: &DbPool) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM cache_metadata WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Check if cache entry is valid
pub async fn is_cache_valid(pool: &DbPool, key: &str) -> sqlx::Result<bool> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM cache_metadata WHERE key = $1 AND expires_at > NOW()")
            .bind(key)
            .fetch_one(pool)
            .await?;
    Ok(count.0 > 0)
}

// =============================================================================
// Statistics queries
// =============================================================================

/// Get repository statistics
pub async fn get_repository_stats(
    pool: &DbPool,
    repository_id: i64,
) -> sqlx::Result<RepositoryStats> {
    let issue_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT state, COUNT(*) FROM issues WHERE repository_id = $1 GROUP BY state",
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    let pr_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT state, COUNT(*) FROM pull_requests WHERE repository_id = $1 GROUP BY state",
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    let release_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM releases WHERE repository_id = $1")
            .bind(repository_id)
            .fetch_one(pool)
            .await?;

    let mut open_issues = 0i64;
    let mut closed_issues = 0i64;
    for (state, count) in issue_counts {
        match state.as_str() {
            "open" => open_issues = count,
            "closed" => closed_issues = count,
            _ => {}
        }
    }

    let mut open_prs = 0i64;
    let mut closed_prs = 0i64;
    for (state, count) in pr_counts {
        match state.as_str() {
            "open" => open_prs = count,
            "closed" => closed_prs = count,
            _ => {}
        }
    }

    // Get merged count separately
    let merged_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pull_requests WHERE repository_id = $1 AND merged = TRUE",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let merged_prs = merged_count.0;

    Ok(RepositoryStats {
        open_issues,
        closed_issues,
        open_prs,
        merged_prs,
        closed_prs,
        releases: release_count.0,
    })
}

/// Repository statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepositoryStats {
    pub open_issues: i64,
    pub closed_issues: i64,
    pub open_prs: i64,
    pub merged_prs: i64,
    pub closed_prs: i64,
    pub releases: i64,
}

// =============================================================================
// Date-range queries (for calendar feature)
// =============================================================================

/// List issues created or closed within a date range
pub async fn list_issues_in_date_range(
    pool: &DbPool,
    repository_id: i64,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<IssueRow>> {
    sqlx::query_as::<_, IssueRow>(
        r#"
        SELECT * FROM issues
        WHERE repository_id = $1
          AND (
            (created_at BETWEEN $2 AND $3)
            OR (closed_at BETWEEN $2 AND $3)
          )
        ORDER BY created_at
        "#,
    )
    .bind(repository_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

/// List milestones with due_on or closed_at within a date range
pub async fn list_milestones_in_date_range(
    pool: &DbPool,
    repository_id: i64,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<MilestoneRow>> {
    sqlx::query_as::<_, MilestoneRow>(
        r#"
        SELECT * FROM milestones
        WHERE repository_id = $1
          AND (
            (due_on BETWEEN $2 AND $3)
            OR (closed_at BETWEEN $2 AND $3)
          )
        ORDER BY COALESCE(due_on, closed_at)
        "#,
    )
    .bind(repository_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

/// List releases published within a date range
pub async fn list_releases_in_date_range(
    pool: &DbPool,
    repository_id: i64,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT * FROM releases
        WHERE repository_id = $1
          AND published_at BETWEEN $2 AND $3
        ORDER BY published_at
        "#,
    )
    .bind(repository_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

/// List pull requests merged within a date range
pub async fn list_pulls_merged_in_date_range(
    pool: &DbPool,
    repository_id: i64,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<PullRequestRow>> {
    sqlx::query_as::<_, PullRequestRow>(
        r#"
        SELECT * FROM pull_requests
        WHERE repository_id = $1
          AND merged_at BETWEEN $2 AND $3
        ORDER BY merged_at
        "#,
    )
    .bind(repository_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

// =============================================================================
// Release plan queries
// =============================================================================

/// List open milestones with due dates up to a forward date
pub async fn list_open_milestones_with_due(
    pool: &DbPool,
    repository_id: i64,
    forward_date: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<MilestoneRow>> {
    sqlx::query_as::<_, MilestoneRow>(
        r#"
        SELECT * FROM milestones
        WHERE repository_id = $1
          AND state = 'open'
          AND due_on IS NOT NULL
          AND due_on <= $2
        ORDER BY due_on ASC
        "#,
    )
    .bind(repository_id)
    .bind(forward_date)
    .fetch_all(pool)
    .await
}

/// Count blocker/critical issues for a milestone
pub async fn count_blocker_issues(
    pool: &DbPool,
    repository_id: i64,
    milestone_id: i64,
) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT i.id) FROM issues i
        JOIN issue_labels il ON il.issue_id = i.id
        WHERE i.repository_id = $1
          AND i.milestone_id = $2
          AND i.state = 'open'
          AND LOWER(il.label_name) IN ('blocker', 'critical')
        "#,
    )
    .bind(repository_id)
    .bind(milestone_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List releases published since a given date
pub async fn list_recent_releases(
    pool: &DbPool,
    repository_id: i64,
    since: chrono::DateTime<Utc>,
) -> sqlx::Result<Vec<ReleaseRow>> {
    sqlx::query_as::<_, ReleaseRow>(
        r#"
        SELECT * FROM releases
        WHERE repository_id = $1
          AND published_at >= $2
        ORDER BY published_at DESC
        "#,
    )
    .bind(repository_id)
    .bind(since)
    .fetch_all(pool)
    .await
}

// =============================================================================
// Organization queries
// =============================================================================

/// Upsert organization
pub async fn upsert_organization(pool: &DbPool, input: &OrganizationInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO organizations (name, base_url)
        VALUES ($1, $2)
        ON CONFLICT (name) DO UPDATE SET
            base_url = EXCLUDED.base_url
        "#,
    )
    .bind(&input.name)
    .bind(&input.base_url)
    .execute(pool)
    .await?;

    Ok(())
}

/// List all organizations
pub async fn list_organizations(pool: &DbPool) -> sqlx::Result<Vec<OrganizationRow>> {
    sqlx::query_as::<_, OrganizationRow>("SELECT * FROM organizations ORDER BY name")
        .fetch_all(pool)
        .await
}

/// Get organization by name
pub async fn get_organization_by_name(
    pool: &DbPool,
    name: &str,
) -> sqlx::Result<Option<OrganizationRow>> {
    sqlx::query_as::<_, OrganizationRow>("SELECT * FROM organizations WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await
}

/// List repositories belonging to a specific organization
pub async fn list_repositories_by_org(
    pool: &DbPool,
    org_name: &str,
) -> sqlx::Result<Vec<RepositoryRow>> {
    sqlx::query_as::<_, RepositoryRow>(
        "SELECT * FROM repositories WHERE org_name = $1 ORDER BY full_name",
    )
    .bind(org_name)
    .fetch_all(pool)
    .await
}

// =============================================================================
// Project queries (GitHub Projects V2)
// =============================================================================

/// Upsert project
pub async fn upsert_project(pool: &DbPool, input: &ProjectInput) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO projects (node_id, number, owner, title, description, url, closed,
                              total_items, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (node_id) DO UPDATE SET
            number = EXCLUDED.number,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            url = EXCLUDED.url,
            closed = EXCLUDED.closed,
            total_items = EXCLUDED.total_items,
            updated_at = EXCLUDED.updated_at,
            synced_at = NOW()
        "#,
    )
    .bind(&input.node_id)
    .bind(input.number)
    .bind(&input.owner)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.url)
    .bind(input.closed)
    .bind(input.total_items)
    .bind(input.created_at)
    .bind(input.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// List projects for an organization
pub async fn list_projects(
    pool: &DbPool,
    owner: &str,
    include_closed: bool,
) -> sqlx::Result<Vec<ProjectRow>> {
    if include_closed {
        sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE owner = $1 ORDER BY number")
            .bind(owner)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, ProjectRow>(
            "SELECT * FROM projects WHERE owner = $1 AND closed = FALSE ORDER BY number",
        )
        .bind(owner)
        .fetch_all(pool)
        .await
    }
}

/// Get project by owner and number
pub async fn get_project(
    pool: &DbPool,
    owner: &str,
    number: i64,
) -> sqlx::Result<Option<ProjectRow>> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE owner = $1 AND number = $2")
        .bind(owner)
        .bind(number)
        .fetch_optional(pool)
        .await
}

/// Get project by node ID
pub async fn get_project_by_node_id(
    pool: &DbPool,
    node_id: &str,
) -> sqlx::Result<Option<ProjectRow>> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE node_id = $1")
        .bind(node_id)
        .fetch_optional(pool)
        .await
}

/// Delete projects not in the current list (stale cleanup)
pub async fn delete_stale_projects(
    pool: &DbPool,
    owner: &str,
    current_node_ids: &[String],
) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM projects WHERE owner = $1 AND node_id != ALL($2)")
        .bind(owner)
        .bind(current_node_ids)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Replace all field definitions for a project (delete + insert)
pub async fn replace_project_fields(
    pool: &DbPool,
    project_id: &str,
    fields: &[ProjectFieldInput],
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM project_fields WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;

    for field in fields {
        sqlx::query(
            r#"
            INSERT INTO project_fields (node_id, project_id, name, field_type, config_json, synced_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(&field.node_id)
        .bind(&field.project_id)
        .bind(&field.name)
        .bind(&field.field_type)
        .bind(&field.config_json)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// List field definitions for a project
pub async fn list_project_fields(
    pool: &DbPool,
    project_id: &str,
) -> sqlx::Result<Vec<ProjectFieldRow>> {
    sqlx::query_as::<_, ProjectFieldRow>(
        "SELECT * FROM project_fields WHERE project_id = $1 ORDER BY name",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Replace all items for a project (delete + insert)
pub async fn replace_project_items(
    pool: &DbPool,
    project_id: &str,
    items: &[ProjectItemInput],
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM project_items WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;

    for item in items {
        sqlx::query(
            r#"
            INSERT INTO project_items (node_id, project_id, content_type, content_number,
                                       content_title, content_state, content_url,
                                       content_repository, content_json, field_values_json,
                                       created_at, updated_at, synced_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            "#,
        )
        .bind(&item.node_id)
        .bind(&item.project_id)
        .bind(&item.content_type)
        .bind(item.content_number)
        .bind(&item.content_title)
        .bind(&item.content_state)
        .bind(&item.content_url)
        .bind(&item.content_repository)
        .bind(&item.content_json)
        .bind(&item.field_values_json)
        .bind(item.created_at)
        .bind(item.updated_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// List items for a project with optional filters
pub async fn list_project_items(
    pool: &DbPool,
    project_id: &str,
    content_type: Option<&str>,
    content_state: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> sqlx::Result<Vec<ProjectItemRow>> {
    let mut query = String::from("SELECT * FROM project_items WHERE project_id = $1");
    let mut param_count = 1;

    if content_type.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND content_type = ${}", param_count));
    }

    if content_state.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND content_state = ${}", param_count));
    }

    query.push_str(" ORDER BY updated_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }
    if let Some(o) = offset {
        query.push_str(&format!(" OFFSET {}", o));
    }

    let mut q = sqlx::query_as::<_, ProjectItemRow>(&query).bind(project_id);

    if let Some(ct) = content_type {
        q = q.bind(ct);
    }
    if let Some(cs) = content_state {
        q = q.bind(cs);
    }

    q.fetch_all(pool).await
}

/// Count project items by status field value
pub async fn count_project_items_by_status(
    pool: &DbPool,
    project_id: &str,
    status_field_name: &str,
) -> sqlx::Result<Vec<(String, i64)>> {
    // Extract status from field_values_json array where field_name matches
    sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT
            COALESCE(fv->>'name', 'No Status') as status,
            COUNT(*) as count
        FROM project_items pi,
             LATERAL (
                SELECT value as fv
                FROM jsonb_array_elements(pi.field_values_json) as value
                WHERE value->>'field_name' = $2
                LIMIT 1
             ) sub
        WHERE pi.project_id = $1
        GROUP BY status
        ORDER BY count DESC
        "#,
    )
    .bind(project_id)
    .bind(status_field_name)
    .fetch_all(pool)
    .await
}

/// Check if data needs refresh based on sync time
pub async fn needs_refresh(
    pool: &DbPool,
    repository_id: i64,
    data_type: &str,
    max_age_seconds: i64,
) -> sqlx::Result<bool> {
    let sync_status = get_sync_status(pool, repository_id).await?;

    let synced_at = match sync_status {
        None => return Ok(true),
        Some(status) => match data_type {
            "issues" => status.issues_synced_at,
            "pulls" => status.pulls_synced_at,
            "releases" => status.releases_synced_at,
            "milestones" => status.milestones_synced_at,
            _ => None,
        },
    };

    match synced_at {
        None => Ok(true),
        Some(dt) => {
            let age = Utc::now().signed_duration_since(dt);
            Ok(age.num_seconds() > max_age_seconds)
        }
    }
}
