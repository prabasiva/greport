//! Release model

use super::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release ID
    pub id: i64,
    /// Git tag name
    pub tag_name: String,
    /// Release name
    pub name: Option<String>,
    /// Release body (markdown)
    pub body: Option<String>,
    /// Is draft release
    pub draft: bool,
    /// Is prerelease
    pub prerelease: bool,
    /// Release author
    pub author: User,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Published timestamp
    pub published_at: Option<DateTime<Utc>>,
}

impl Release {
    /// Get display name (name or tag_name)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.tag_name)
    }

    /// Check if release is published
    pub fn is_published(&self) -> bool {
        !self.draft && self.published_at.is_some()
    }

    /// Check if this is a stable release (not draft, not prerelease)
    pub fn is_stable(&self) -> bool {
        !self.draft && !self.prerelease
    }
}
