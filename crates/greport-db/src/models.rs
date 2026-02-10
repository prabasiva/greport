//! Database models matching the PostgreSQL schema

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Repository record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RepositoryRow {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub default_branch: String,
    pub org_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: DateTime<Utc>,
}

/// Milestone record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MilestoneRow {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub due_on: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// Issue record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueRow {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub milestone_id: Option<i64>,
    pub author_login: String,
    pub author_id: i64,
    pub comments_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by_login: Option<String>,
    pub synced_at: DateTime<Utc>,
}

/// Issue label association
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueLabelRow {
    pub issue_id: i64,
    pub label_id: i64,
    pub label_name: String,
    pub label_color: Option<String>,
}

/// Issue assignee association
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueAssigneeRow {
    pub issue_id: i64,
    pub user_id: i64,
    pub user_login: String,
}

/// Pull request record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PullRequestRow {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub draft: bool,
    pub milestone_id: Option<i64>,
    pub author_login: String,
    pub author_id: i64,
    pub head_ref: String,
    pub base_ref: String,
    pub merged: bool,
    pub merged_at: Option<DateTime<Utc>>,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// Release record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReleaseRow {
    pub id: i64,
    pub repository_id: i64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub author_login: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// API key record
#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub owner: String,
    pub scopes: Vec<String>,
    pub rate_limit: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

/// Sync status record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SyncStatusRow {
    pub repository_id: i64,
    pub issues_synced_at: Option<DateTime<Utc>>,
    pub pulls_synced_at: Option<DateTime<Utc>>,
    pub releases_synced_at: Option<DateTime<Utc>>,
    pub milestones_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
}

/// Cache metadata record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CacheMetadataRow {
    pub key: String,
    pub data_type: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub hit_count: i32,
}

/// Input for creating/updating a repository
#[derive(Debug, Clone)]
pub struct RepositoryInput {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub default_branch: String,
    pub org_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating/updating a milestone
#[derive(Debug, Clone)]
pub struct MilestoneInput {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub due_on: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Input for creating/updating an issue
#[derive(Debug, Clone)]
pub struct IssueInput {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub milestone_id: Option<i64>,
    pub author_login: String,
    pub author_id: i64,
    pub comments_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by_login: Option<String>,
}

/// Input for creating/updating a pull request
#[derive(Debug, Clone)]
pub struct PullRequestInput {
    pub id: i64,
    pub repository_id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub draft: bool,
    pub milestone_id: Option<i64>,
    pub author_login: String,
    pub author_id: i64,
    pub head_ref: String,
    pub base_ref: String,
    pub merged: bool,
    pub merged_at: Option<DateTime<Utc>>,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Input for creating/updating a release
#[derive(Debug, Clone)]
pub struct ReleaseInput {
    pub id: i64,
    pub repository_id: i64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub author_login: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Organization record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OrganizationRow {
    pub id: Uuid,
    pub name: String,
    pub base_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Input for creating/updating an organization
#[derive(Debug, Clone)]
pub struct OrganizationInput {
    pub name: String,
    pub base_url: Option<String>,
}

/// Input for creating an API key
#[derive(Debug, Clone)]
pub struct ApiKeyInput {
    pub name: String,
    pub key_hash: String,
    pub owner: String,
    pub scopes: Vec<String>,
    pub rate_limit: i32,
    pub expires_at: Option<DateTime<Utc>>,
}
