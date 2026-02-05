//! User model

use serde::{Deserialize, Serialize};

/// GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: i64,
    /// Username
    pub login: String,
    /// Avatar URL
    pub avatar_url: String,
    /// Profile URL
    pub html_url: String,
}

impl User {
    /// Create a placeholder user for unknown authors
    pub fn unknown() -> Self {
        Self {
            id: 0,
            login: "unknown".to_string(),
            avatar_url: String::new(),
            html_url: String::new(),
        }
    }
}
