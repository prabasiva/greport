//! Repository model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: i64,
    /// Owner (user or organization)
    pub owner: String,
    /// Repository name
    pub name: String,
    /// Full name (owner/repo)
    pub full_name: String,
    /// Description
    pub description: Option<String>,
    /// Is private repository
    pub private: bool,
    /// Default branch name
    pub default_branch: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}
