//! Database queries

use crate::models::*;
use crate::DbPool;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Repository queries

/// Get repository by full name
pub async fn get_repository_by_name(
    pool: &DbPool,
    full_name: &str,
) -> sqlx::Result<Option<RepositoryRow>> {
    sqlx::query_as::<_, RepositoryRow>(
        "SELECT * FROM repositories WHERE full_name = $1"
    )
    .bind(full_name)
    .fetch_optional(pool)
    .await
}

/// Upsert repository
pub async fn upsert_repository(pool: &DbPool, repo: &RepositoryRow) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO repositories (id, owner, name, full_name, description, private,
                                  default_branch, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT (full_name) DO UPDATE SET
            description = EXCLUDED.description,
            private = EXCLUDED.private,
            default_branch = EXCLUDED.default_branch,
            updated_at = EXCLUDED.updated_at,
            synced_at = NOW()
        "#,
    )
    .bind(repo.id)
    .bind(&repo.owner)
    .bind(&repo.name)
    .bind(&repo.full_name)
    .bind(&repo.description)
    .bind(repo.private)
    .bind(&repo.default_branch)
    .bind(repo.created_at)
    .bind(repo.updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

// Issue queries

/// Get issues by repository
pub async fn get_issues_by_repo(
    pool: &DbPool,
    repo_id: i64,
    state: Option<&str>,
) -> sqlx::Result<Vec<IssueRow>> {
    match state {
        Some(s) => {
            sqlx::query_as::<_, IssueRow>(
                "SELECT * FROM issues WHERE repo_id = $1 AND state = $2 ORDER BY created_at DESC",
            )
            .bind(repo_id)
            .bind(s)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, IssueRow>(
                "SELECT * FROM issues WHERE repo_id = $1 ORDER BY created_at DESC",
            )
            .bind(repo_id)
            .fetch_all(pool)
            .await
        }
    }
}

/// Upsert issue
pub async fn upsert_issue(pool: &DbPool, issue: &IssueRow) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO issues (id, repo_id, number, title, body, state, author_login,
                           created_at, updated_at, closed_at, comments_count, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
        ON CONFLICT (repo_id, number) DO UPDATE SET
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            updated_at = EXCLUDED.updated_at,
            closed_at = EXCLUDED.closed_at,
            comments_count = EXCLUDED.comments_count,
            synced_at = NOW()
        "#,
    )
    .bind(issue.id)
    .bind(issue.repo_id)
    .bind(issue.number)
    .bind(&issue.title)
    .bind(&issue.body)
    .bind(&issue.state)
    .bind(&issue.author_login)
    .bind(issue.created_at)
    .bind(issue.updated_at)
    .bind(issue.closed_at)
    .bind(issue.comments_count)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get stale issues (not synced recently)
pub async fn get_stale_issues(
    pool: &DbPool,
    repo_id: i64,
    stale_threshold: DateTime<Utc>,
) -> sqlx::Result<Vec<IssueRow>> {
    sqlx::query_as::<_, IssueRow>(
        "SELECT * FROM issues WHERE repo_id = $1 AND synced_at < $2",
    )
    .bind(repo_id)
    .bind(stale_threshold)
    .fetch_all(pool)
    .await
}

// Saved report queries

/// Get saved reports for user
pub async fn get_saved_reports(pool: &DbPool, user_id: &str) -> sqlx::Result<Vec<SavedReportRow>> {
    sqlx::query_as::<_, SavedReportRow>(
        "SELECT * FROM saved_reports WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get saved report by ID
pub async fn get_saved_report(pool: &DbPool, id: Uuid) -> sqlx::Result<Option<SavedReportRow>> {
    sqlx::query_as::<_, SavedReportRow>("SELECT * FROM saved_reports WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Create saved report
pub async fn create_saved_report(
    pool: &DbPool,
    user_id: &str,
    name: &str,
    report_type: &str,
    config: serde_json::Value,
) -> sqlx::Result<SavedReportRow> {
    sqlx::query_as::<_, SavedReportRow>(
        r#"
        INSERT INTO saved_reports (user_id, name, report_type, config, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(report_type)
    .bind(config)
    .fetch_one(pool)
    .await
}

/// Delete saved report
pub async fn delete_saved_report(pool: &DbPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM saved_reports WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// API key queries

/// Get API key by hash
pub async fn get_api_key_by_hash(pool: &DbPool, key_hash: &str) -> sqlx::Result<Option<ApiKeyRow>> {
    sqlx::query_as::<_, ApiKeyRow>(
        "SELECT * FROM api_keys WHERE key_hash = $1 AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await
}

/// Update API key last used
pub async fn update_api_key_last_used(pool: &DbPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
