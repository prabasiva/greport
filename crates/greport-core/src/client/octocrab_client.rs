//! Octocrab-based GitHub client implementation

use super::{
    GitHubClient, IssueParams, IssueStateFilter, PullParams, PullStateFilter, RateLimitInfo, RepoId,
};
use crate::models::{
    Issue, IssueEvent, IssueState, Label, Milestone, MilestoneState, PullRequest, PullState,
    Release, Repository, Review, User,
};
use crate::{Error, Result};
use async_trait::async_trait;
use octocrab::Octocrab;
use tracing::{debug, instrument};

/// GitHub client using octocrab
pub struct OctocrabClient {
    client: Octocrab,
}

impl OctocrabClient {
    /// Create a new client with the given token and optional base URL
    ///
    /// # Arguments
    /// * `token` - GitHub personal access token
    /// * `base_url` - Optional base URL for GitHub Enterprise (e.g., `https://github.mycompany.com/api/v3`)
    pub fn new(token: &str, base_url: Option<&str>) -> Result<Self> {
        let mut builder = Octocrab::builder().personal_token(token.to_string());

        // Log token type (without exposing the actual token)
        let token_type = if token.starts_with("ghp_") {
            "classic PAT"
        } else if token.starts_with("github_pat_") {
            "fine-grained PAT"
        } else if token.starts_with("gho_") {
            "OAuth token"
        } else {
            "unknown token type"
        };
        debug!(token_type = token_type, "Creating GitHub client");

        if let Some(url) = base_url {
            debug!(base_url = url, "Using GitHub Enterprise base URL");
            builder = builder
                .base_uri(url)
                .map_err(|e| Error::Custom(format!("Invalid base URL '{}': {}", url, e)))?;
        } else {
            debug!("Using default GitHub.com API (https://api.github.com)");
        }

        let client = builder
            .build()
            .map_err(|e| Error::Custom(format!("Failed to create GitHub client: {}", e)))?;

        debug!("GitHub client created successfully");
        Ok(Self { client })
    }

    /// Create a client with only a token (uses default GitHub.com API)
    pub fn with_token(token: &str) -> Result<Self> {
        Self::new(token, None)
    }

    /// Create a client from environment variables
    ///
    /// Reads:
    /// * `GITHUB_TOKEN` - Required: GitHub personal access token
    /// * `GITHUB_BASE_URL` - Optional: Base URL for GitHub Enterprise
    pub fn from_env() -> Result<Self> {
        debug!("Creating GitHub client from environment variables");
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| Error::MissingToken)?;
        debug!("GITHUB_TOKEN found in environment");

        let base_url = std::env::var("GITHUB_BASE_URL").ok();
        if base_url.is_some() {
            debug!("GITHUB_BASE_URL found in environment");
        }

        Self::new(&token, base_url.as_deref())
    }

    /// Create a client from a Config object
    pub fn from_config(config: &crate::config::Config) -> Result<Self> {
        debug!("Creating GitHub client from config");
        let token = config.github_token()?;
        Self::new(&token, config.github.base_url.as_deref())
    }

    /// Convert octocrab issue to our Issue model
    fn convert_issue(issue: octocrab::models::issues::Issue) -> Issue {
        Issue {
            id: issue.id.0 as i64,
            number: issue.number,
            title: issue.title,
            body: issue.body,
            state: if issue.state == octocrab::models::IssueState::Open {
                IssueState::Open
            } else {
                IssueState::Closed
            },
            labels: issue
                .labels
                .into_iter()
                .map(|l| Label {
                    id: l.id.0 as i64,
                    name: l.name,
                    color: l.color,
                    description: l.description,
                })
                .collect(),
            assignees: issue
                .assignees
                .into_iter()
                .map(Self::convert_user)
                .collect(),
            milestone: issue.milestone.map(Self::convert_milestone),
            author: Self::convert_user(issue.user),
            comments_count: issue.comments,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            closed_by: None, // Would need additional API call
        }
    }

    /// Convert octocrab user to our User model
    fn convert_user(user: octocrab::models::Author) -> User {
        User {
            id: user.id.0 as i64,
            login: user.login,
            avatar_url: user.avatar_url.to_string(),
            html_url: user.html_url.to_string(),
        }
    }

    /// Convert octocrab milestone to our Milestone model
    fn convert_milestone(ms: octocrab::models::Milestone) -> Milestone {
        Milestone {
            id: ms.id.0 as i64,
            number: ms.number as u64,
            title: ms.title,
            description: ms.description,
            state: if ms.state.as_deref() == Some("open") {
                MilestoneState::Open
            } else {
                MilestoneState::Closed
            },
            open_issues: ms.open_issues.unwrap_or(0) as u32,
            closed_issues: ms.closed_issues.unwrap_or(0) as u32,
            due_on: ms.due_on,
            created_at: ms.created_at,
            closed_at: None,
        }
    }
}

#[async_trait]
impl GitHubClient for OctocrabClient {
    #[instrument(skip(self), fields(repo = %repo))]
    async fn get_repository(&self, repo: &RepoId) -> Result<Repository> {
        debug!("Fetching repository info");

        let r = self.client.repos(&repo.owner, &repo.name).get().await?;

        Ok(Repository {
            id: r.id.0 as i64,
            owner: repo.owner.clone(),
            name: r.name,
            full_name: r.full_name.unwrap_or_else(|| repo.full_name()),
            description: r.description,
            private: r.private.unwrap_or(false),
            default_branch: r.default_branch.unwrap_or_else(|| "main".to_string()),
            created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: r.updated_at.unwrap_or_else(chrono::Utc::now),
        })
    }

    async fn list_org_repos(&self, org: &str) -> Result<Vec<Repository>> {
        debug!("Fetching organization repositories");

        let page = self
            .client
            .orgs(org)
            .list_repos()
            .per_page(100)
            .send()
            .await?;

        let repos = page
            .items
            .into_iter()
            .map(|r| Repository {
                id: r.id.0 as i64,
                owner: org.to_string(),
                name: r.name.clone(),
                full_name: r.full_name.unwrap_or_else(|| format!("{}/{}", org, r.name)),
                description: r.description,
                private: r.private.unwrap_or(false),
                default_branch: r.default_branch.unwrap_or_else(|| "main".to_string()),
                created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
                updated_at: r.updated_at.unwrap_or_else(chrono::Utc::now),
            })
            .collect();

        Ok(repos)
    }

    #[instrument(skip(self), fields(repo = %repo))]
    async fn list_issues(&self, repo: &RepoId, params: IssueParams) -> Result<Vec<Issue>> {
        debug!("Fetching issues");

        let state = match params.state {
            IssueStateFilter::Open => octocrab::params::State::Open,
            IssueStateFilter::Closed => octocrab::params::State::Closed,
            IssueStateFilter::All => octocrab::params::State::All,
        };

        let issues_handler = self.client.issues(&repo.owner, &repo.name);
        let mut builder = issues_handler
            .list()
            .state(state)
            .per_page(params.per_page.min(100) as u8);

        if let Some(labels) = &params.labels {
            builder = builder.labels(labels);
        }

        if let Some(assignee) = &params.assignee {
            builder = builder.assignee(assignee.as_str());
        }

        if let Some(since) = params.since {
            builder = builder.since(since);
        }

        let mut all_issues = Vec::new();
        let mut page = builder.send().await?;

        loop {
            for issue in page.items {
                // Skip pull requests (GitHub API includes them in issues)
                if issue.pull_request.is_none() {
                    all_issues.push(Self::convert_issue(issue));
                }
            }

            page = match self
                .client
                .get_page::<octocrab::models::issues::Issue>(&page.next)
                .await?
            {
                Some(next) => next,
                None => break,
            };
        }

        debug!(count = all_issues.len(), "Fetched issues");
        Ok(all_issues)
    }

    async fn get_issue(&self, repo: &RepoId, number: u64) -> Result<Issue> {
        let issue = self
            .client
            .issues(&repo.owner, &repo.name)
            .get(number)
            .await?;

        Ok(Self::convert_issue(issue))
    }

    async fn list_issue_events(&self, repo: &RepoId, number: u64) -> Result<Vec<IssueEvent>> {
        // Use the REST API directly for issue events since octocrab doesn't have list_events
        let route = format!(
            "/repos/{}/{}/issues/{}/events?per_page=100",
            repo.owner, repo.name, number
        );

        #[derive(serde::Deserialize)]
        struct EventLabel {
            name: String,
        }

        #[derive(serde::Deserialize)]
        struct ApiEvent {
            id: i64,
            event: String,
            actor: Option<octocrab::models::Author>,
            created_at: chrono::DateTime<chrono::Utc>,
            label: Option<EventLabel>,
            assignee: Option<octocrab::models::Author>,
        }

        let events: Vec<ApiEvent> = self.client.get(&route, None::<&()>).await?;

        Ok(events
            .into_iter()
            .map(|e| IssueEvent {
                id: e.id,
                event_type: e.event,
                actor: e.actor.map(Self::convert_user),
                created_at: e.created_at,
                label_name: e.label.map(|l| l.name),
                assignee: e.assignee.map(Self::convert_user),
            })
            .collect())
    }

    async fn list_milestones(&self, repo: &RepoId) -> Result<Vec<Milestone>> {
        // Use the REST API directly for milestones
        let route = format!(
            "/repos/{}/{}/milestones?state=all&per_page=100",
            repo.owner, repo.name
        );
        let milestones: Vec<octocrab::models::Milestone> =
            self.client.get(&route, None::<&()>).await?;

        Ok(milestones
            .into_iter()
            .map(Self::convert_milestone)
            .collect())
    }

    #[instrument(skip(self), fields(repo = %repo))]
    async fn list_pulls(&self, repo: &RepoId, params: PullParams) -> Result<Vec<PullRequest>> {
        debug!("Fetching pull requests");

        let state = match params.state {
            PullStateFilter::Open => octocrab::params::State::Open,
            PullStateFilter::Closed => octocrab::params::State::Closed,
            PullStateFilter::All => octocrab::params::State::All,
        };

        let page = self
            .client
            .pulls(&repo.owner, &repo.name)
            .list()
            .state(state)
            .per_page(params.per_page.min(100) as u8)
            .send()
            .await?;

        let prs = page
            .items
            .into_iter()
            .map(|pr| {
                let is_open = pr
                    .state
                    .as_ref()
                    .map(|s| format!("{:?}", s).to_lowercase().contains("open"))
                    .unwrap_or(false);
                PullRequest {
                    id: pr.id.0 as i64,
                    number: pr.number,
                    title: pr.title.unwrap_or_default(),
                    body: pr.body,
                    state: if is_open {
                        PullState::Open
                    } else {
                        PullState::Closed
                    },
                    draft: pr.draft.unwrap_or(false),
                    author: pr
                        .user
                        .map(|u| Self::convert_user(*u))
                        .unwrap_or_else(|| User {
                            id: 0,
                            login: "unknown".to_string(),
                            avatar_url: String::new(),
                            html_url: String::new(),
                        }),
                    labels: pr
                        .labels
                        .unwrap_or_default()
                        .into_iter()
                        .map(|l| Label {
                            id: l.id.0 as i64,
                            name: l.name,
                            color: l.color,
                            description: l.description,
                        })
                        .collect(),
                    milestone: pr.milestone.map(|m| Self::convert_milestone(*m)),
                    head_ref: pr.head.ref_field,
                    base_ref: pr.base.ref_field,
                    merged: pr.merged_at.is_some(),
                    merged_at: pr.merged_at,
                    additions: 0,     // Would need additional API call
                    deletions: 0,     // Would need additional API call
                    changed_files: 0, // Would need additional API call
                    created_at: pr.created_at.unwrap_or_else(chrono::Utc::now),
                    updated_at: pr.updated_at.unwrap_or_else(chrono::Utc::now),
                    closed_at: pr.closed_at,
                }
            })
            .collect();

        Ok(prs)
    }

    async fn get_pull(&self, repo: &RepoId, number: u64) -> Result<PullRequest> {
        let pr = self
            .client
            .pulls(&repo.owner, &repo.name)
            .get(number)
            .await?;

        let is_open = pr
            .state
            .as_ref()
            .map(|s| format!("{:?}", s).to_lowercase().contains("open"))
            .unwrap_or(false);
        Ok(PullRequest {
            id: pr.id.0 as i64,
            number: pr.number,
            title: pr.title.unwrap_or_default(),
            body: pr.body,
            state: if is_open {
                PullState::Open
            } else {
                PullState::Closed
            },
            draft: pr.draft.unwrap_or(false),
            author: pr
                .user
                .map(|u| Self::convert_user(*u))
                .unwrap_or_else(|| User {
                    id: 0,
                    login: "unknown".to_string(),
                    avatar_url: String::new(),
                    html_url: String::new(),
                }),
            labels: pr
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(|l| Label {
                    id: l.id.0 as i64,
                    name: l.name,
                    color: l.color,
                    description: l.description,
                })
                .collect(),
            milestone: pr.milestone.map(|m| Self::convert_milestone(*m)),
            head_ref: pr.head.ref_field,
            base_ref: pr.base.ref_field,
            merged: pr.merged_at.is_some(),
            merged_at: pr.merged_at,
            additions: pr.additions.unwrap_or(0) as u32,
            deletions: pr.deletions.unwrap_or(0) as u32,
            changed_files: pr.changed_files.unwrap_or(0) as u32,
            created_at: pr.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: pr.updated_at.unwrap_or_else(chrono::Utc::now),
            closed_at: pr.closed_at,
        })
    }

    async fn list_pull_reviews(&self, repo: &RepoId, number: u64) -> Result<Vec<Review>> {
        let reviews = self
            .client
            .pulls(&repo.owner, &repo.name)
            .list_reviews(number)
            .send()
            .await?;

        Ok(reviews
            .items
            .into_iter()
            .map(|r| Review {
                id: r.id.0 as i64,
                user: r.user.map(Self::convert_user),
                body: r.body,
                state: r.state.map(|s| format!("{:?}", s)).unwrap_or_default(),
                submitted_at: r.submitted_at,
            })
            .collect())
    }

    async fn list_releases(&self, repo: &RepoId) -> Result<Vec<Release>> {
        let releases = self
            .client
            .repos(&repo.owner, &repo.name)
            .releases()
            .list()
            .per_page(100)
            .send()
            .await?;

        Ok(releases
            .items
            .into_iter()
            .map(|r| Release {
                id: r.id.0 as i64,
                tag_name: r.tag_name,
                name: r.name,
                body: r.body,
                draft: r.draft,
                prerelease: r.prerelease,
                author: r.author.map(Self::convert_user).unwrap_or_else(|| User {
                    id: 0,
                    login: "unknown".to_string(),
                    avatar_url: String::new(),
                    html_url: String::new(),
                }),
                created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
                published_at: r.published_at,
            })
            .collect())
    }

    async fn get_release(&self, repo: &RepoId, tag: &str) -> Result<Release> {
        let r = self
            .client
            .repos(&repo.owner, &repo.name)
            .releases()
            .get_by_tag(tag)
            .await?;

        Ok(Release {
            id: r.id.0 as i64,
            tag_name: r.tag_name,
            name: r.name,
            body: r.body,
            draft: r.draft,
            prerelease: r.prerelease,
            author: r.author.map(Self::convert_user).unwrap_or_else(|| User {
                id: 0,
                login: "unknown".to_string(),
                avatar_url: String::new(),
                html_url: String::new(),
            }),
            created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
            published_at: r.published_at,
        })
    }

    async fn get_user(&self, username: &str) -> Result<User> {
        let user = self.client.users(username).profile().await?;

        Ok(User {
            id: user.id.0 as i64,
            login: user.login,
            avatar_url: user.avatar_url.to_string(),
            html_url: user.html_url.to_string(),
        })
    }

    async fn rate_limit(&self) -> Result<RateLimitInfo> {
        let rate_limit = self.client.ratelimit().get().await?;

        Ok(RateLimitInfo {
            remaining: rate_limit.resources.core.remaining as u32,
            limit: rate_limit.resources.core.limit as u32,
            reset: rate_limit.resources.core.reset as u64,
        })
    }
}
