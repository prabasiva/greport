//! Octocrab-based GitHub client implementation

use super::graphql::GraphQLClient;
use super::retry::RetryConfig;
use super::{
    GitHubClient, IssueParams, IssueStateFilter, ProjectClient, PullParams, PullStateFilter,
    RateLimitInfo, RepoId,
};
use crate::models::{
    Issue, IssueEvent, IssueState, Label, Milestone, MilestoneState, Project, ProjectItem,
    PullRequest, PullState, Release, Repository, Review, User,
};
use crate::{Error, Result};
use async_trait::async_trait;
use octocrab::Octocrab;
use tracing::{debug, error, info, instrument, warn};

/// Log detailed error information for debugging
fn log_api_error(operation: &str, endpoint: &str, err: &octocrab::Error) {
    error!(
        operation = operation,
        endpoint = endpoint,
        "GitHub API request failed"
    );

    match err {
        octocrab::Error::GitHub {
            source,
            backtrace: _,
        } => {
            error!(
                status_code = %source.status_code,
                message = %source.message,
                documentation_url = source.documentation_url.as_deref().unwrap_or("none"),
                "GitHub API error response"
            );
            if let Some(errors) = &source.errors {
                for (i, e) in errors.iter().enumerate() {
                    error!(
                        error_index = i,
                        resource = e
                            .get("resource")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown"),
                        field = e.get("field").and_then(|v| v.as_str()).unwrap_or("unknown"),
                        code = e.get("code").and_then(|v| v.as_str()).unwrap_or("unknown"),
                        "GitHub API error detail"
                    );
                }
            }
        }
        octocrab::Error::Http {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "HTTP",
                details = %source,
                "HTTP transport error"
            );
        }
        octocrab::Error::Serde {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "Serde",
                details = %source,
                "JSON parsing error - API may have returned HTML error page or unexpected format"
            );
            warn!("This often indicates: 1) Invalid API URL, 2) Authentication failure, 3) Network proxy/firewall interference");
        }
        octocrab::Error::InvalidHeaderValue {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "InvalidHeader",
                details = %source,
                "Invalid HTTP header value"
            );
        }
        octocrab::Error::Uri {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "URL",
                details = %source,
                "Invalid URL"
            );
        }
        octocrab::Error::Service {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "Service",
                details = %source,
                "Service error"
            );
        }
        octocrab::Error::Other {
            source,
            backtrace: _,
        } => {
            error!(
                error_type = "Other",
                details = %source,
                "Other error"
            );
        }
        _ => {
            error!(error_type = "Unknown", details = %err, "Unknown error type");
        }
    }
}

/// GitHub client using octocrab with retry support
pub struct OctocrabClient {
    client: Octocrab,
    retry_config: RetryConfig,
    graphql: GraphQLClient,
}

impl std::fmt::Debug for OctocrabClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OctocrabClient")
            .field("retry_config", &self.retry_config)
            .field("graphql", &self.graphql)
            .finish_non_exhaustive()
    }
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

        let graphql = GraphQLClient::new(token, base_url)?;

        debug!("GitHub client created successfully");
        Ok(Self {
            client,
            retry_config: RetryConfig::default(),
            graphql,
        })
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
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
        let endpoint = format!("/repos/{}/{}", repo.owner, repo.name);
        info!(endpoint = %endpoint, "Fetching repository info");

        let r = match self.client.repos(&repo.owner, &repo.name).get().await {
            Ok(r) => {
                debug!(
                    repo_id = r.id.0,
                    full_name = r.full_name.as_deref().unwrap_or("unknown"),
                    private = r.private.unwrap_or(false),
                    "Successfully fetched repository"
                );
                r
            }
            Err(e) => {
                log_api_error("get_repository", &endpoint, &e);
                return Err(e.into());
            }
        };

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
        let endpoint = format!("/orgs/{}/repos", org);
        info!(endpoint = %endpoint, org = org, "Fetching organization repositories");

        let page = match self
            .client
            .orgs(org)
            .list_repos()
            .per_page(100)
            .send()
            .await
        {
            Ok(p) => {
                debug!(
                    repos_count = p.items.len(),
                    "Fetched organization repositories"
                );
                p
            }
            Err(e) => {
                log_api_error("list_org_repos", &endpoint, &e);
                return Err(e.into());
            }
        };

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
        let endpoint = format!("/repos/{}/{}/issues", repo.owner, repo.name);
        info!(
            endpoint = %endpoint,
            state = ?params.state,
            labels = ?params.labels,
            assignee = ?params.assignee,
            per_page = params.per_page,
            "Starting to fetch issues"
        );

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
            debug!(labels = ?labels, "Filtering by labels");
            builder = builder.labels(labels);
        }

        if let Some(assignee) = &params.assignee {
            debug!(assignee = %assignee, "Filtering by assignee");
            builder = builder.assignee(assignee.as_str());
        }

        if let Some(since) = params.since {
            debug!(since = %since, "Filtering by since date");
            builder = builder.since(since);
        }

        debug!("Sending initial issues request");
        let mut all_issues = Vec::new();
        let mut page = match builder.send().await {
            Ok(p) => {
                debug!(
                    items_in_page = p.items.len(),
                    has_next = p.next.is_some(),
                    "Received first page of issues"
                );
                p
            }
            Err(e) => {
                log_api_error("list_issues", &endpoint, &e);
                return Err(e.into());
            }
        };

        let mut page_num = 1;
        loop {
            debug!(
                page = page_num,
                items = page.items.len(),
                "Processing issues page"
            );

            for issue in page.items {
                // Skip pull requests (GitHub API includes them in issues)
                if issue.pull_request.is_none() {
                    all_issues.push(Self::convert_issue(issue));
                }
            }

            if page.next.is_none() {
                debug!("No more pages, finished fetching issues");
                break;
            }

            page_num += 1;
            debug!(page = page_num, "Fetching next page of issues");

            page = match self
                .client
                .get_page::<octocrab::models::issues::Issue>(&page.next)
                .await
            {
                Ok(Some(next)) => {
                    debug!(
                        items_in_page = next.items.len(),
                        has_next = next.next.is_some(),
                        "Received next page of issues"
                    );
                    next
                }
                Ok(None) => {
                    debug!("No more pages available");
                    break;
                }
                Err(e) => {
                    log_api_error("list_issues (pagination)", &endpoint, &e);
                    return Err(e.into());
                }
            };
        }

        info!(
            total_issues = all_issues.len(),
            pages_fetched = page_num,
            "Completed fetching issues"
        );
        Ok(all_issues)
    }

    #[instrument(skip(self), fields(repo = %repo, issue_number = number))]
    async fn get_issue(&self, repo: &RepoId, number: u64) -> Result<Issue> {
        let endpoint = format!("/repos/{}/{}/issues/{}", repo.owner, repo.name, number);
        info!(endpoint = %endpoint, "Fetching single issue");

        let issue = match self
            .client
            .issues(&repo.owner, &repo.name)
            .get(number)
            .await
        {
            Ok(i) => {
                debug!(
                    issue_id = i.id.0,
                    title = %i.title,
                    state = ?i.state,
                    "Successfully fetched issue"
                );
                i
            }
            Err(e) => {
                log_api_error("get_issue", &endpoint, &e);
                return Err(e.into());
            }
        };

        Ok(Self::convert_issue(issue))
    }

    #[instrument(skip(self), fields(repo = %repo, issue_number = number))]
    async fn list_issue_events(&self, repo: &RepoId, number: u64) -> Result<Vec<IssueEvent>> {
        // Use the REST API directly for issue events since octocrab doesn't have list_events
        let route = format!(
            "/repos/{}/{}/issues/{}/events?per_page=100",
            repo.owner, repo.name, number
        );
        info!(endpoint = %route, "Fetching issue events");

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

        let events: Vec<ApiEvent> = match self
            .client
            .get::<Vec<ApiEvent>, _, _>(&route, None::<&()>)
            .await
        {
            Ok(e) => {
                debug!(events_count = e.len(), "Successfully fetched issue events");
                e
            }
            Err(e) => {
                log_api_error("list_issue_events", &route, &e);
                return Err(e.into());
            }
        };

        let result: Vec<IssueEvent> = events
            .into_iter()
            .map(|e| IssueEvent {
                id: e.id,
                event_type: e.event,
                actor: e.actor.map(Self::convert_user),
                created_at: e.created_at,
                label_name: e.label.map(|l| l.name),
                assignee: e.assignee.map(Self::convert_user),
            })
            .collect();

        info!(
            total_events = result.len(),
            "Completed fetching issue events"
        );
        Ok(result)
    }

    #[instrument(skip(self), fields(repo = %repo))]
    async fn list_milestones(&self, repo: &RepoId) -> Result<Vec<Milestone>> {
        // Use the REST API directly for milestones
        let route = format!(
            "/repos/{}/{}/milestones?state=all&per_page=100",
            repo.owner, repo.name
        );
        info!(endpoint = %route, "Fetching milestones");

        let milestones: Vec<octocrab::models::Milestone> = match self
            .client
            .get::<Vec<octocrab::models::Milestone>, _, _>(&route, None::<&()>)
            .await
        {
            Ok(m) => {
                debug!(
                    milestones_count = m.len(),
                    "Successfully fetched milestones"
                );
                m
            }
            Err(e) => {
                log_api_error("list_milestones", &route, &e);
                return Err(e.into());
            }
        };

        let result: Vec<Milestone> = milestones
            .into_iter()
            .map(Self::convert_milestone)
            .collect();

        info!(
            total_milestones = result.len(),
            "Completed fetching milestones"
        );
        Ok(result)
    }

    #[instrument(skip(self), fields(repo = %repo))]
    async fn list_pulls(&self, repo: &RepoId, params: PullParams) -> Result<Vec<PullRequest>> {
        let endpoint = format!("/repos/{}/{}/pulls", repo.owner, repo.name);
        info!(
            endpoint = %endpoint,
            state = ?params.state,
            per_page = params.per_page,
            "Starting to fetch pull requests"
        );

        let state = match params.state {
            PullStateFilter::Open => octocrab::params::State::Open,
            PullStateFilter::Closed => octocrab::params::State::Closed,
            PullStateFilter::All => octocrab::params::State::All,
        };

        debug!("Sending pull requests request");
        let page = match self
            .client
            .pulls(&repo.owner, &repo.name)
            .list()
            .state(state)
            .per_page(params.per_page.min(100) as u8)
            .send()
            .await
        {
            Ok(p) => {
                debug!(
                    items_in_page = p.items.len(),
                    has_next = p.next.is_some(),
                    "Successfully fetched pull requests page"
                );
                p
            }
            Err(e) => {
                log_api_error("list_pulls", &endpoint, &e);
                return Err(e.into());
            }
        };

        let prs: Vec<PullRequest> = page
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

        info!(total_prs = prs.len(), "Completed fetching pull requests");
        Ok(prs)
    }

    #[instrument(skip(self), fields(repo = %repo, pr_number = number))]
    async fn get_pull(&self, repo: &RepoId, number: u64) -> Result<PullRequest> {
        let endpoint = format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number);
        info!(endpoint = %endpoint, "Fetching single pull request");

        let pr = match self.client.pulls(&repo.owner, &repo.name).get(number).await {
            Ok(p) => {
                debug!(
                    pr_id = p.id.0,
                    title = p.title.as_deref().unwrap_or("untitled"),
                    state = ?p.state,
                    draft = p.draft.unwrap_or(false),
                    "Successfully fetched pull request"
                );
                p
            }
            Err(e) => {
                log_api_error("get_pull", &endpoint, &e);
                return Err(e.into());
            }
        };

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

    #[instrument(skip(self), fields(repo = %repo, pr_number = number))]
    async fn list_pull_reviews(&self, repo: &RepoId, number: u64) -> Result<Vec<Review>> {
        let endpoint = format!(
            "/repos/{}/{}/pulls/{}/reviews",
            repo.owner, repo.name, number
        );
        info!(endpoint = %endpoint, "Fetching pull request reviews");

        let reviews = match self
            .client
            .pulls(&repo.owner, &repo.name)
            .list_reviews(number)
            .send()
            .await
        {
            Ok(r) => {
                debug!(
                    reviews_count = r.items.len(),
                    "Successfully fetched reviews"
                );
                r
            }
            Err(e) => {
                log_api_error("list_pull_reviews", &endpoint, &e);
                return Err(e.into());
            }
        };

        let result: Vec<Review> = reviews
            .items
            .into_iter()
            .map(|r| Review {
                id: r.id.0 as i64,
                user: r.user.map(Self::convert_user),
                body: r.body,
                state: r.state.map(|s| format!("{:?}", s)).unwrap_or_default(),
                submitted_at: r.submitted_at,
            })
            .collect();

        info!(total_reviews = result.len(), "Completed fetching reviews");
        Ok(result)
    }

    #[instrument(skip(self), fields(repo = %repo))]
    async fn list_releases(&self, repo: &RepoId) -> Result<Vec<Release>> {
        let endpoint = format!("/repos/{}/{}/releases", repo.owner, repo.name);
        info!(endpoint = %endpoint, "Fetching releases");

        let releases = match self
            .client
            .repos(&repo.owner, &repo.name)
            .releases()
            .list()
            .per_page(100)
            .send()
            .await
        {
            Ok(r) => {
                debug!(
                    releases_count = r.items.len(),
                    "Successfully fetched releases"
                );
                r
            }
            Err(e) => {
                log_api_error("list_releases", &endpoint, &e);
                return Err(e.into());
            }
        };

        let result: Vec<Release> = releases
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
            .collect();

        info!(total_releases = result.len(), "Completed fetching releases");
        Ok(result)
    }

    #[instrument(skip(self), fields(repo = %repo, tag = %tag))]
    async fn get_release(&self, repo: &RepoId, tag: &str) -> Result<Release> {
        let endpoint = format!("/repos/{}/{}/releases/tags/{}", repo.owner, repo.name, tag);
        info!(endpoint = %endpoint, "Fetching release by tag");

        let r = match self
            .client
            .repos(&repo.owner, &repo.name)
            .releases()
            .get_by_tag(tag)
            .await
        {
            Ok(rel) => {
                debug!(
                    release_id = rel.id.0,
                    tag_name = %rel.tag_name,
                    name = rel.name.as_deref().unwrap_or("unnamed"),
                    draft = rel.draft,
                    prerelease = rel.prerelease,
                    "Successfully fetched release"
                );
                rel
            }
            Err(e) => {
                log_api_error("get_release", &endpoint, &e);
                return Err(e.into());
            }
        };

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

    #[instrument(skip(self), fields(username = %username))]
    async fn get_user(&self, username: &str) -> Result<User> {
        let endpoint = format!("/users/{}", username);
        info!(endpoint = %endpoint, "Fetching user profile");

        let user = match self.client.users(username).profile().await {
            Ok(u) => {
                debug!(
                    user_id = u.id.0,
                    login = %u.login,
                    "Successfully fetched user profile"
                );
                u
            }
            Err(e) => {
                log_api_error("get_user", &endpoint, &e);
                return Err(e.into());
            }
        };

        Ok(User {
            id: user.id.0 as i64,
            login: user.login,
            avatar_url: user.avatar_url.to_string(),
            html_url: user.html_url.to_string(),
        })
    }

    #[instrument(skip(self))]
    async fn rate_limit(&self) -> Result<RateLimitInfo> {
        let endpoint = "/rate_limit";
        info!(endpoint = %endpoint, "Fetching rate limit status");

        let rate_limit = match self.client.ratelimit().get().await {
            Ok(rl) => {
                debug!(
                    remaining = rl.resources.core.remaining,
                    limit = rl.resources.core.limit,
                    reset = %rl.resources.core.reset,
                    "Successfully fetched rate limit"
                );
                rl
            }
            Err(e) => {
                log_api_error("rate_limit", endpoint, &e);
                return Err(e.into());
            }
        };

        Ok(RateLimitInfo {
            remaining: rate_limit.resources.core.remaining as u32,
            limit: rate_limit.resources.core.limit as u32,
            reset: rate_limit.resources.core.reset,
        })
    }
}

#[async_trait]
impl ProjectClient for OctocrabClient {
    async fn list_projects(&self, org: &str) -> Result<Vec<Project>> {
        self.graphql.list_org_projects(org).await
    }

    async fn get_project(&self, org: &str, project_number: u64) -> Result<Project> {
        self.graphql.get_project_detail(org, project_number).await
    }

    async fn list_project_items(&self, project_node_id: &str) -> Result<Vec<ProjectItem>> {
        self.graphql.list_items(project_node_id).await
    }
}
