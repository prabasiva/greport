//! Mock GitHub client for testing

use super::{GitHubClient, IssueParams, PullParams, RateLimitInfo, RepoId};
use crate::models::{
    Issue, IssueEvent, IssueState, Label, Milestone, MilestoneState, PullRequest, PullState,
    Release, Repository, Review, User,
};
use crate::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Mock data store for testing
#[derive(Default, Clone)]
pub struct MockData {
    pub repositories: HashMap<String, Repository>,
    pub issues: HashMap<String, Vec<Issue>>,
    pub issue_events: HashMap<(String, u64), Vec<IssueEvent>>,
    pub milestones: HashMap<String, Vec<Milestone>>,
    pub pulls: HashMap<String, Vec<PullRequest>>,
    pub pull_reviews: HashMap<(String, u64), Vec<Review>>,
    pub releases: HashMap<String, Vec<Release>>,
    pub users: HashMap<String, User>,
}

impl MockData {
    /// Create a new empty mock data store
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a repository to the mock store
    pub fn with_repository(mut self, repo: Repository) -> Self {
        let key = format!("{}/{}", repo.owner, repo.name);
        self.repositories.insert(key, repo);
        self
    }

    /// Add issues to a repository
    pub fn with_issues(mut self, repo: &str, issues: Vec<Issue>) -> Self {
        self.issues.insert(repo.to_string(), issues);
        self
    }

    /// Add milestones to a repository
    pub fn with_milestones(mut self, repo: &str, milestones: Vec<Milestone>) -> Self {
        self.milestones.insert(repo.to_string(), milestones);
        self
    }

    /// Add pull requests to a repository
    pub fn with_pulls(mut self, repo: &str, pulls: Vec<PullRequest>) -> Self {
        self.pulls.insert(repo.to_string(), pulls);
        self
    }

    /// Add releases to a repository
    pub fn with_releases(mut self, repo: &str, releases: Vec<Release>) -> Self {
        self.releases.insert(repo.to_string(), releases);
        self
    }

    /// Add a user
    pub fn with_user(mut self, user: User) -> Self {
        self.users.insert(user.login.clone(), user);
        self
    }
}

/// Mock GitHub client for testing
pub struct MockGitHubClient {
    data: Arc<RwLock<MockData>>,
}

impl MockGitHubClient {
    /// Create a new mock client with the given data
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }

    /// Create a mock client with sample test data
    pub fn with_sample_data() -> Self {
        let now = chrono::Utc::now();
        let one_day_ago = now - chrono::Duration::days(1);
        let one_week_ago = now - chrono::Duration::days(7);
        let two_weeks_ago = now - chrono::Duration::days(14);

        let test_user = User {
            id: 1,
            login: "test-user".to_string(),
            avatar_url: "https://example.com/avatar.png".to_string(),
            html_url: "https://github.com/test-user".to_string(),
        };

        let bug_label = Label {
            id: 1,
            name: "bug".to_string(),
            color: "d73a4a".to_string(),
            description: Some("Something isn't working".to_string()),
        };

        let feature_label = Label {
            id: 2,
            name: "enhancement".to_string(),
            color: "a2eeef".to_string(),
            description: Some("New feature or request".to_string()),
        };

        let milestone = Milestone {
            id: 1,
            number: 1,
            title: "v1.0".to_string(),
            description: Some("First release".to_string()),
            state: MilestoneState::Open,
            open_issues: 3,
            closed_issues: 2,
            due_on: Some(now + chrono::Duration::days(30)),
            created_at: two_weeks_ago,
            closed_at: None,
        };

        let issues = vec![
            Issue {
                id: 1,
                number: 1,
                title: "Fix login bug".to_string(),
                body: Some("Users cannot log in".to_string()),
                state: IssueState::Open,
                labels: vec![bug_label.clone()],
                assignees: vec![test_user.clone()],
                milestone: Some(milestone.clone()),
                author: test_user.clone(),
                comments_count: 5,
                created_at: two_weeks_ago,
                updated_at: one_day_ago,
                closed_at: None,
                closed_by: None,
            },
            Issue {
                id: 2,
                number: 2,
                title: "Add dark mode".to_string(),
                body: Some("Implement dark mode support".to_string()),
                state: IssueState::Open,
                labels: vec![feature_label.clone()],
                assignees: vec![],
                milestone: Some(milestone.clone()),
                author: test_user.clone(),
                comments_count: 2,
                created_at: one_week_ago,
                updated_at: one_week_ago,
                closed_at: None,
                closed_by: None,
            },
            Issue {
                id: 3,
                number: 3,
                title: "Update documentation".to_string(),
                body: Some("Docs are outdated".to_string()),
                state: IssueState::Closed,
                labels: vec![],
                assignees: vec![test_user.clone()],
                milestone: None,
                author: test_user.clone(),
                comments_count: 1,
                created_at: two_weeks_ago,
                updated_at: one_day_ago,
                closed_at: Some(one_day_ago),
                closed_by: Some(test_user.clone()),
            },
        ];

        let pulls = vec![
            PullRequest {
                id: 1,
                number: 10,
                title: "Fix login authentication".to_string(),
                body: Some("Fixes #1".to_string()),
                state: PullState::Open,
                draft: false,
                author: test_user.clone(),
                labels: vec![bug_label.clone()],
                milestone: Some(milestone.clone()),
                head_ref: "fix-login".to_string(),
                base_ref: "main".to_string(),
                merged: false,
                merged_at: None,
                additions: 50,
                deletions: 10,
                changed_files: 3,
                created_at: one_week_ago,
                updated_at: one_day_ago,
                closed_at: None,
            },
            PullRequest {
                id: 2,
                number: 11,
                title: "Add user settings page".to_string(),
                body: Some("New settings feature".to_string()),
                state: PullState::Closed,
                draft: false,
                author: test_user.clone(),
                labels: vec![feature_label.clone()],
                milestone: None,
                head_ref: "user-settings".to_string(),
                base_ref: "main".to_string(),
                merged: true,
                merged_at: Some(one_day_ago),
                additions: 200,
                deletions: 20,
                changed_files: 8,
                created_at: two_weeks_ago,
                updated_at: one_day_ago,
                closed_at: Some(one_day_ago),
            },
        ];

        let releases = vec![
            Release {
                id: 1,
                tag_name: "v0.1.0".to_string(),
                name: Some("Initial Release".to_string()),
                body: Some("First release of the project".to_string()),
                draft: false,
                prerelease: false,
                author: test_user.clone(),
                created_at: two_weeks_ago,
                published_at: Some(two_weeks_ago),
            },
        ];

        let repo = Repository {
            id: 1,
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            full_name: "test-owner/test-repo".to_string(),
            description: Some("A test repository".to_string()),
            private: false,
            default_branch: "main".to_string(),
            created_at: two_weeks_ago,
            updated_at: now,
        };

        let data = MockData::new()
            .with_repository(repo)
            .with_issues("test-owner/test-repo", issues)
            .with_milestones("test-owner/test-repo", vec![milestone])
            .with_pulls("test-owner/test-repo", pulls)
            .with_releases("test-owner/test-repo", releases)
            .with_user(test_user);

        Self::new(data)
    }

    /// Get mutable access to the mock data
    pub fn data_mut(&self) -> std::sync::RwLockWriteGuard<'_, MockData> {
        self.data.write().unwrap()
    }

    /// Get read access to the mock data
    pub fn data(&self) -> std::sync::RwLockReadGuard<'_, MockData> {
        self.data.read().unwrap()
    }
}

#[async_trait]
impl GitHubClient for MockGitHubClient {
    async fn get_repository(&self, repo: &RepoId) -> Result<Repository> {
        let data = self.data.read().unwrap();
        data.repositories
            .get(&repo.full_name())
            .cloned()
            .ok_or_else(|| crate::Error::NotFound(format!("Repository {} not found", repo)))
    }

    async fn list_org_repos(&self, org: &str) -> Result<Vec<Repository>> {
        let data = self.data.read().unwrap();
        let repos: Vec<Repository> = data
            .repositories
            .values()
            .filter(|r| r.owner == org)
            .cloned()
            .collect();
        Ok(repos)
    }

    async fn list_issues(&self, repo: &RepoId, params: IssueParams) -> Result<Vec<Issue>> {
        let data = self.data.read().unwrap();
        let issues = data
            .issues
            .get(&repo.full_name())
            .cloned()
            .unwrap_or_default();

        // Apply filters
        let filtered: Vec<Issue> = issues
            .into_iter()
            .filter(|i| {
                use super::IssueStateFilter;
                match params.state {
                    IssueStateFilter::Open => i.state == IssueState::Open,
                    IssueStateFilter::Closed => i.state == IssueState::Closed,
                    IssueStateFilter::All => true,
                }
            })
            .filter(|i| {
                if let Some(ref labels) = params.labels {
                    labels.iter().all(|l| i.labels.iter().any(|il| il.name == *l))
                } else {
                    true
                }
            })
            .filter(|i| {
                if let Some(ref assignee) = params.assignee {
                    i.assignees.iter().any(|a| &a.login == assignee)
                } else {
                    true
                }
            })
            .filter(|i| {
                if let Some(since) = params.since {
                    i.updated_at >= since
                } else {
                    true
                }
            })
            .collect();

        Ok(filtered)
    }

    async fn get_issue(&self, repo: &RepoId, number: u64) -> Result<Issue> {
        let data = self.data.read().unwrap();
        let issues = data
            .issues
            .get(&repo.full_name())
            .ok_or_else(|| crate::Error::NotFound(format!("Repository {} not found", repo)))?;

        issues
            .iter()
            .find(|i| i.number == number)
            .cloned()
            .ok_or_else(|| crate::Error::NotFound(format!("Issue #{} not found", number)))
    }

    async fn list_issue_events(&self, repo: &RepoId, number: u64) -> Result<Vec<IssueEvent>> {
        let data = self.data.read().unwrap();
        Ok(data
            .issue_events
            .get(&(repo.full_name(), number))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_milestones(&self, repo: &RepoId) -> Result<Vec<Milestone>> {
        let data = self.data.read().unwrap();
        Ok(data
            .milestones
            .get(&repo.full_name())
            .cloned()
            .unwrap_or_default())
    }

    async fn list_pulls(&self, repo: &RepoId, params: PullParams) -> Result<Vec<PullRequest>> {
        let data = self.data.read().unwrap();
        let pulls = data
            .pulls
            .get(&repo.full_name())
            .cloned()
            .unwrap_or_default();

        let filtered: Vec<PullRequest> = pulls
            .into_iter()
            .filter(|p| {
                use super::PullStateFilter;
                match params.state {
                    PullStateFilter::Open => p.state == PullState::Open,
                    PullStateFilter::Closed => p.state == PullState::Closed,
                    PullStateFilter::All => true,
                }
            })
            .collect();

        Ok(filtered)
    }

    async fn get_pull(&self, repo: &RepoId, number: u64) -> Result<PullRequest> {
        let data = self.data.read().unwrap();
        let pulls = data
            .pulls
            .get(&repo.full_name())
            .ok_or_else(|| crate::Error::NotFound(format!("Repository {} not found", repo)))?;

        pulls
            .iter()
            .find(|p| p.number == number)
            .cloned()
            .ok_or_else(|| crate::Error::NotFound(format!("PR #{} not found", number)))
    }

    async fn list_pull_reviews(&self, repo: &RepoId, number: u64) -> Result<Vec<Review>> {
        let data = self.data.read().unwrap();
        Ok(data
            .pull_reviews
            .get(&(repo.full_name(), number))
            .cloned()
            .unwrap_or_default())
    }

    async fn list_releases(&self, repo: &RepoId) -> Result<Vec<Release>> {
        let data = self.data.read().unwrap();
        Ok(data
            .releases
            .get(&repo.full_name())
            .cloned()
            .unwrap_or_default())
    }

    async fn get_release(&self, repo: &RepoId, tag: &str) -> Result<Release> {
        let data = self.data.read().unwrap();
        let releases = data
            .releases
            .get(&repo.full_name())
            .ok_or_else(|| crate::Error::NotFound(format!("Repository {} not found", repo)))?;

        releases
            .iter()
            .find(|r| r.tag_name == tag)
            .cloned()
            .ok_or_else(|| crate::Error::NotFound(format!("Release {} not found", tag)))
    }

    async fn get_user(&self, username: &str) -> Result<User> {
        let data = self.data.read().unwrap();
        data.users
            .get(username)
            .cloned()
            .ok_or_else(|| crate::Error::NotFound(format!("User {} not found", username)))
    }

    async fn rate_limit(&self) -> Result<RateLimitInfo> {
        Ok(RateLimitInfo {
            remaining: 5000,
            limit: 5000,
            reset: chrono::Utc::now().timestamp() as u64 + 3600,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_get_repository() {
        let client = MockGitHubClient::with_sample_data();
        let repo_id = RepoId::new("test-owner", "test-repo");

        let repo = client.get_repository(&repo_id).await.unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner, "test-owner");
    }

    #[tokio::test]
    async fn test_mock_client_list_issues() {
        let client = MockGitHubClient::with_sample_data();
        let repo_id = RepoId::new("test-owner", "test-repo");

        let issues = client
            .list_issues(&repo_id, IssueParams::default())
            .await
            .unwrap();
        assert_eq!(issues.len(), 2); // Only open issues by default
    }

    #[tokio::test]
    async fn test_mock_client_list_all_issues() {
        let client = MockGitHubClient::with_sample_data();
        let repo_id = RepoId::new("test-owner", "test-repo");

        let mut params = IssueParams::default();
        params.state = super::super::IssueStateFilter::All;

        let issues = client.list_issues(&repo_id, params).await.unwrap();
        assert_eq!(issues.len(), 3);
    }

    #[tokio::test]
    async fn test_mock_client_list_pulls() {
        let client = MockGitHubClient::with_sample_data();
        let repo_id = RepoId::new("test-owner", "test-repo");

        let pulls = client
            .list_pulls(&repo_id, PullParams::default())
            .await
            .unwrap();
        assert_eq!(pulls.len(), 1); // Only open PRs by default
    }

    #[tokio::test]
    async fn test_mock_client_rate_limit() {
        let client = MockGitHubClient::with_sample_data();

        let rate_limit = client.rate_limit().await.unwrap();
        assert_eq!(rate_limit.remaining, 5000);
        assert_eq!(rate_limit.limit, 5000);
    }
}
