//! GitHub client registry for multi-organization support

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tracing::{debug, info, warn};

use crate::client::GitHubClient;
use crate::client::OctocrabClient;
use crate::client::RepoId;
use crate::config::Config;
use crate::{Error, Result};

/// Summary metadata about a configured organization.
#[derive(Debug, Clone, Serialize)]
pub struct OrgEntry {
    /// Organization name
    pub name: String,
    /// GitHub API base URL (None = public github.com)
    pub base_url: Option<String>,
    /// Number of configured repos
    pub repo_count: usize,
    /// Configured repository names
    pub repo_names: Vec<String>,
    /// Whether a token is configured
    pub has_token: bool,
}

/// Registry that manages multiple GitHub clients, one per organization.
///
/// The registry resolves the correct client for a given organization or
/// repository. It supports:
/// - Exact org name matching
/// - Case-insensitive fallback matching
/// - Default client fallback for unconfigured orgs
pub struct GitHubClientRegistry {
    clients: HashMap<String, Arc<OctocrabClient>>,
    default_client: Option<Arc<OctocrabClient>>,
    org_entries: Vec<OrgEntry>,
}

impl GitHubClientRegistry {
    /// Create a registry with only a default client (no per-org entries).
    pub fn with_default(client: OctocrabClient) -> Self {
        Self {
            clients: HashMap::new(),
            default_client: Some(Arc::new(client)),
            org_entries: Vec::new(),
        }
    }

    /// Build a registry from the application config.
    ///
    /// Creates one `OctocrabClient` per `[[organizations]]` entry and
    /// optionally a default client from `[github]` token.
    pub fn from_config(config: &Config) -> Result<Self> {
        let mut clients = HashMap::new();

        for org in &config.organizations {
            debug!(org = %org.name, "Creating client for organization");
            let client = OctocrabClient::new(&org.token, org.base_url.as_deref())?;
            clients.insert(org.name.to_lowercase(), Arc::new(client));
        }

        let default_client = match config.github_token() {
            Ok(token) => {
                debug!("Creating default client from [github] config");
                let client = OctocrabClient::new(&token, config.github.base_url.as_deref())?;
                Some(Arc::new(client))
            }
            Err(_) => {
                if clients.is_empty() {
                    return Err(Error::MissingToken);
                }
                warn!("No default [github] token configured; only org-specific tokens available");
                None
            }
        };

        info!(
            org_count = clients.len(),
            has_default = default_client.is_some(),
            "GitHub client registry initialized"
        );

        let org_entries: Vec<OrgEntry> = config
            .organizations
            .iter()
            .map(|org| {
                let repos = org.repos.as_deref().unwrap_or_default();
                OrgEntry {
                    name: org.name.clone(),
                    base_url: org.base_url.clone(),
                    repo_count: repos.len(),
                    repo_names: repos.iter().map(|r| r.to_string()).collect(),
                    has_token: !org.token.is_empty(),
                }
            })
            .collect();

        Ok(Self {
            clients,
            default_client,
            org_entries,
        })
    }

    /// Get the client for a specific organization.
    ///
    /// Resolution order:
    /// 1. Exact (case-insensitive) match in configured organizations
    /// 2. Fallback to default client
    /// 3. Error if no match and no default
    pub fn client_for_org(&self, org: &str) -> Result<&Arc<OctocrabClient>> {
        let org_lower = org.to_lowercase();

        // Exact case-insensitive match
        if let Some(client) = self.clients.get(&org_lower) {
            debug!(org = org, "Found org-specific client");
            return Ok(client);
        }

        // Fallback to default
        if let Some(ref default) = self.default_client {
            debug!(org = org, "Using default client for unconfigured org");
            return Ok(default);
        }

        let env_var = org.to_uppercase().replace('-', "_");
        Err(Error::OrgNotConfigured {
            org: org.to_string(),
            env_var,
        })
    }

    /// Get the client for a repository, resolving by its owner (organization).
    pub fn client_for_repo(&self, repo: &RepoId) -> Result<&Arc<OctocrabClient>> {
        self.client_for_org(&repo.owner)
    }

    /// Get the default client (from `[github]` config).
    pub fn default_client(&self) -> Result<&Arc<OctocrabClient>> {
        self.default_client.as_ref().ok_or(Error::MissingToken)
    }

    /// List all configured organization names.
    pub fn org_names(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a specific organization is configured.
    pub fn has_org(&self, org: &str) -> bool {
        self.clients.contains_key(&org.to_lowercase())
    }

    /// Get metadata entries for all configured organizations.
    pub fn org_entries(&self) -> &[OrgEntry] {
        &self.org_entries
    }

    /// Validate all configured tokens by calling the GitHub rate_limit API.
    ///
    /// Returns the count of tokens that validated successfully.
    /// Failures are logged as warnings but are non-fatal.
    pub async fn validate_tokens(&self) -> usize {
        let mut valid = 0usize;

        // Validate default client
        if let Some(ref client) = self.default_client {
            match client.rate_limit().await {
                Ok(info) => {
                    info!(
                        remaining = info.remaining,
                        limit = info.limit,
                        "Default token validated"
                    );
                    valid += 1;
                }
                Err(e) => {
                    warn!(error = %e, "Default token validation failed");
                }
            }
        }

        // Validate per-org clients
        for (org_name, client) in &self.clients {
            match client.rate_limit().await {
                Ok(info) => {
                    info!(org = %org_name, remaining = info.remaining, limit = info.limit, "Org token validated");
                    valid += 1;
                }
                Err(e) => {
                    warn!(org = %org_name, error = %e, "Org token validation failed");
                }
            }
        }

        valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, GitHubConfig, OrgConfig};

    /// Helper to build a Config with org entries.
    /// Note: Tests that call `from_config` will actually create OctocrabClient
    /// instances which just build an HTTP client -- they don't make network calls.
    fn config_with_orgs(
        default_token: Option<&str>,
        orgs: Vec<(&str, &str, Option<&str>)>,
    ) -> Config {
        Config {
            github: GitHubConfig {
                token: default_token.map(|t| t.to_string()),
                base_url: None,
            },
            organizations: orgs
                .into_iter()
                .map(|(name, token, base_url)| OrgConfig {
                    name: name.to_string(),
                    token: token.to_string(),
                    base_url: base_url.map(|u| u.to_string()),
                    repos: None,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_registry_client_for_known_org() {
        let config = config_with_orgs(
            Some("ghp_default"),
            vec![
                ("org-alpha", "ghp_alpha", None),
                ("org-beta", "ghp_beta", None),
            ],
        );

        let registry = GitHubClientRegistry::from_config(&config).unwrap();

        // Should succeed for configured orgs
        assert!(registry.client_for_org("org-alpha").is_ok());
        assert!(registry.client_for_org("org-beta").is_ok());

        // Different Arc pointers for different orgs
        let alpha = registry.client_for_org("org-alpha").unwrap();
        let beta = registry.client_for_org("org-beta").unwrap();
        assert!(!Arc::ptr_eq(alpha, beta));
    }

    #[tokio::test]
    async fn test_registry_client_fallback() {
        let config = config_with_orgs(Some("ghp_default"), vec![("org-alpha", "ghp_alpha", None)]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();

        // Unknown org should fall back to default
        let unknown = registry.client_for_org("unknown-org").unwrap();
        let default = registry.default_client().unwrap();
        assert!(Arc::ptr_eq(unknown, default));
    }

    #[tokio::test]
    async fn test_registry_no_token_error() {
        let config = config_with_orgs(None, vec![("org-alpha", "ghp_alpha", None)]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();

        // No default client available
        assert!(registry.default_client().is_err());

        // Unknown org with no default should error
        let result = registry.client_for_org("unknown-org");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::OrgNotConfigured { org, env_var } => {
                assert_eq!(org, "unknown-org");
                assert_eq!(env_var, "UNKNOWN_ORG");
            }
            other => panic!("Expected OrgNotConfigured, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_registry_case_insensitive() {
        let config = config_with_orgs(Some("ghp_default"), vec![("My-Org", "ghp_myorg", None)]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();

        // All case variants should resolve to the same client
        let lower = registry.client_for_org("my-org").unwrap();
        let upper = registry.client_for_org("MY-ORG").unwrap();
        let mixed = registry.client_for_org("My-Org").unwrap();
        assert!(Arc::ptr_eq(lower, upper));
        assert!(Arc::ptr_eq(lower, mixed));
    }

    #[tokio::test]
    async fn test_registry_org_names() {
        let config = config_with_orgs(
            Some("ghp_default"),
            vec![
                ("org-alpha", "ghp_alpha", None),
                ("org-beta", "ghp_beta", None),
            ],
        );

        let registry = GitHubClientRegistry::from_config(&config).unwrap();
        let mut names = registry.org_names();
        names.sort();
        assert_eq!(names, vec!["org-alpha", "org-beta"]);
    }

    #[tokio::test]
    async fn test_registry_from_config_backward_compat() {
        // Config with only [github] and no organizations
        let config = config_with_orgs(Some("ghp_default"), vec![]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();

        assert!(registry.org_names().is_empty());
        assert!(registry.default_client().is_ok());

        // Any org falls back to default
        assert!(registry.client_for_org("any-org").is_ok());
    }

    #[tokio::test]
    async fn test_registry_no_tokens_at_all() {
        // No default, no orgs -- should fail
        let config = config_with_orgs(None, vec![]);
        let result = GitHubClientRegistry::from_config(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_has_org() {
        let config = config_with_orgs(Some("ghp_default"), vec![("org-alpha", "ghp_alpha", None)]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();
        assert!(registry.has_org("org-alpha"));
        assert!(registry.has_org("ORG-ALPHA"));
        assert!(!registry.has_org("org-beta"));
    }

    #[tokio::test]
    async fn test_registry_client_for_repo() {
        let config = config_with_orgs(Some("ghp_default"), vec![("org-alpha", "ghp_alpha", None)]);

        let registry = GitHubClientRegistry::from_config(&config).unwrap();
        let repo = RepoId::new("org-alpha", "my-repo");
        let client = registry.client_for_repo(&repo).unwrap();
        let org_client = registry.client_for_org("org-alpha").unwrap();
        assert!(Arc::ptr_eq(client, org_client));
    }
}
