//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Cached repository
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RepositoryRow {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: DateTime<Utc>,
}

/// Cached issue
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueRow {
    pub id: i64,
    pub repo_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author_login: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub comments_count: i32,
    pub synced_at: DateTime<Utc>,
}

/// Cached pull request
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PullRequestRow {
    pub id: i64,
    pub repo_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub draft: bool,
    pub author_login: String,
    pub head_ref: String,
    pub base_ref: String,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub synced_at: DateTime<Utc>,
}

/// Issue label association
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueLabelRow {
    pub issue_id: i64,
    pub label_name: String,
    pub label_color: Option<String>,
}

/// Issue assignee association
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IssueAssigneeRow {
    pub issue_id: i64,
    pub assignee_login: String,
}

/// Saved report configuration
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SavedReportRow {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: serde_json::Value,
    pub schedule: Option<String>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API key for authentication
#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
