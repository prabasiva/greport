//! Error types for greport-core

use thiserror::Error;

/// Result type alias using greport Error
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for greport operations
#[derive(Error, Debug)]
pub enum Error {
    /// GitHub API error
    #[error("GitHub API error: {0}")]
    GitHubApi(Box<octocrab::Error>),

    /// Invalid repository format
    #[error("Invalid repository format: {0}. Expected 'owner/repo'")]
    InvalidRepoFormat(String),

    /// Repository not found
    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    /// Missing GitHub token
    #[error("Missing GitHub token. Set GITHUB_TOKEN environment variable")]
    MissingToken,

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Resets at {reset_at}")]
    RateLimitExceeded {
        /// Time when rate limit resets
        reset_at: String,
    },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid date range
    #[error("Invalid date range: start ({start}) must be before end ({end})")]
    InvalidDateRange {
        /// Start date
        start: String,
        /// End date
        end: String,
    },

    /// Milestone not found
    #[error("Milestone not found: {0}")]
    MilestoneNotFound(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a custom error with a message
    pub fn custom(msg: impl Into<String>) -> Self {
        Error::Custom(msg.into())
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::Network(_) | Error::RateLimitExceeded { .. })
    }
}

impl From<octocrab::Error> for Error {
    fn from(err: octocrab::Error) -> Self {
        Error::GitHubApi(Box::new(err))
    }
}
