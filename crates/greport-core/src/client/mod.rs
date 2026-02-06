//! GitHub API client abstraction

mod octocrab_client;
mod params;
mod retry;

#[cfg(any(test, feature = "mock"))]
mod mock_client;

pub use retry::RetryConfig;

pub use octocrab_client::OctocrabClient;
pub use params::*;

#[cfg(any(test, feature = "mock"))]
pub use mock_client::{MockData, MockGitHubClient};

use crate::models::{Issue, IssueEvent, Milestone, PullRequest, Release, Repository, Review, User};
use crate::Result;
use async_trait::async_trait;

/// Repository identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoId {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub name: String,
}

impl RepoId {
    /// Create a new RepoId
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// Parse a full repository name (owner/repo)
    pub fn parse(full_name: &str) -> Result<Self> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(crate::Error::InvalidRepoFormat(full_name.to_string()));
        }
        Ok(Self::new(parts[0], parts[1]))
    }

    /// Get the full repository name
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

impl std::fmt::Display for RepoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

/// Trait for GitHub API operations
#[async_trait]
pub trait GitHubClient: Send + Sync {
    // Repository operations

    /// Get repository information
    async fn get_repository(&self, repo: &RepoId) -> Result<Repository>;

    /// List repositories for an organization
    async fn list_org_repos(&self, org: &str) -> Result<Vec<Repository>>;

    // Issue operations

    /// List issues for a repository
    async fn list_issues(&self, repo: &RepoId, params: IssueParams) -> Result<Vec<Issue>>;

    /// Get a single issue
    async fn get_issue(&self, repo: &RepoId, number: u64) -> Result<Issue>;

    /// List events for an issue
    async fn list_issue_events(&self, repo: &RepoId, number: u64) -> Result<Vec<IssueEvent>>;

    /// List milestones for a repository
    async fn list_milestones(&self, repo: &RepoId) -> Result<Vec<Milestone>>;

    // Pull request operations

    /// List pull requests for a repository
    async fn list_pulls(&self, repo: &RepoId, params: PullParams) -> Result<Vec<PullRequest>>;

    /// Get a single pull request
    async fn get_pull(&self, repo: &RepoId, number: u64) -> Result<PullRequest>;

    /// List reviews for a pull request
    async fn list_pull_reviews(&self, repo: &RepoId, number: u64) -> Result<Vec<Review>>;

    // Release operations

    /// List releases for a repository
    async fn list_releases(&self, repo: &RepoId) -> Result<Vec<Release>>;

    /// Get a single release by tag
    async fn get_release(&self, repo: &RepoId, tag: &str) -> Result<Release>;

    // User operations

    /// Get user information
    async fn get_user(&self, username: &str) -> Result<User>;

    // Rate limit

    /// Get current rate limit status
    async fn rate_limit(&self) -> Result<RateLimitInfo>;
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Remaining requests
    pub remaining: u32,
    /// Total limit
    pub limit: u32,
    /// Reset time (Unix timestamp)
    pub reset: u64,
}
