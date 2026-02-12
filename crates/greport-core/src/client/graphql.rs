//! Lightweight GraphQL client for GitHub Projects V2 API.
//!
//! This module provides a thin HTTP wrapper for issuing GraphQL queries
//! against the GitHub API. It handles authentication, endpoint resolution
//! (including GitHub Enterprise), and JSON deserialization.

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, warn};

use crate::models::{
    FieldValue, IterationValue, LabelInfo, Project, ProjectField, ProjectFieldType,
    ProjectFieldValue, ProjectItem, ProjectItemContent, SelectOption,
};
use crate::{Error, Result};

// ---------------------------------------------------------------------------
// GraphQL query constants
// ---------------------------------------------------------------------------

/// List all projects for an organization.
const LIST_ORG_PROJECTS: &str = r#"
query($org: String!, $first: Int!, $after: String) {
  organization(login: $org) {
    projectsV2(first: $first, after: $after, orderBy: {field: UPDATED_AT, direction: DESC}) {
      nodes {
        id
        number
        title
        shortDescription
        url
        closed
        createdAt
        updatedAt
        items { totalCount }
      }
      pageInfo { hasNextPage endCursor }
    }
  }
}
"#;

/// Get a single project with field definitions.
const GET_PROJECT_WITH_FIELDS: &str = r#"
query($org: String!, $number: Int!) {
  organization(login: $org) {
    projectV2(number: $number) {
      id
      number
      title
      shortDescription
      url
      closed
      createdAt
      updatedAt
      items { totalCount }
      fields(first: 30) {
        nodes {
          ... on ProjectV2FieldCommon {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
              color
              description
            }
          }
          ... on ProjectV2IterationField {
            id
            name
            dataType
            configuration {
              iterations {
                id
                title
                startDate
                duration
              }
            }
          }
        }
      }
    }
  }
}
"#;

/// List items in a project (paginated).
const LIST_PROJECT_ITEMS: &str = r#"
query($nodeId: ID!, $first: Int!, $after: String) {
  node(id: $nodeId) {
    ... on ProjectV2 {
      items(first: $first, after: $after) {
        nodes {
          id
          createdAt
          updatedAt
          content {
            ... on Issue {
              number
              title
              state
              url
              repository { nameWithOwner }
              assignees(first: 10) { nodes { login } }
              labels(first: 10) { nodes { name color } }
            }
            ... on PullRequest {
              number
              title
              state
              url
              merged
              repository { nameWithOwner }
              author { login }
            }
            ... on DraftIssue {
              title
              body
              assignees(first: 10) { nodes { login } }
            }
          }
          fieldValues(first: 20) {
            nodes {
              ... on ProjectV2ItemFieldTextValue {
                text
                field { ... on ProjectV2FieldCommon { name } }
              }
              ... on ProjectV2ItemFieldNumberValue {
                number
                field { ... on ProjectV2FieldCommon { name } }
              }
              ... on ProjectV2ItemFieldDateValue {
                date
                field { ... on ProjectV2FieldCommon { name } }
              }
              ... on ProjectV2ItemFieldSingleSelectValue {
                name
                optionId
                field { ... on ProjectV2FieldCommon { name } }
              }
              ... on ProjectV2ItemFieldIterationValue {
                title
                startDate
                duration
                iterationId
                field { ... on ProjectV2FieldCommon { name } }
              }
            }
          }
        }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}
"#;

// ---------------------------------------------------------------------------
// Endpoint resolution
// ---------------------------------------------------------------------------

/// Derive the GraphQL endpoint URL from an optional REST API base URL.
///
/// Resolution rules:
///   None                            -> https://api.github.com/graphql
///   https://ghe.co/api/v3          -> https://ghe.co/api/graphql
///   https://ghe.co/api             -> https://ghe.co/api/graphql
///   https://ghe.co                 -> https://ghe.co/api/graphql
pub fn derive_graphql_url(base_url: Option<&str>) -> String {
    match base_url {
        None => "https://api.github.com/graphql".to_string(),
        Some(url) => {
            let trimmed = url.trim_end_matches('/');
            let base = trimmed
                .strip_suffix("/api/v3")
                .or_else(|| trimmed.strip_suffix("/api"))
                .unwrap_or(trimmed);
            format!("{}/api/graphql", base)
        }
    }
}

// ---------------------------------------------------------------------------
// GraphQL client
// ---------------------------------------------------------------------------

/// Lightweight GraphQL client for GitHub APIs.
pub struct GraphQLClient {
    http: Client,
    endpoint: String,
    token: String,
}

impl std::fmt::Debug for GraphQLClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQLClient")
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    error_type: Option<String>,
}

impl GraphQLClient {
    /// Create a new GraphQL client.
    pub fn new(token: &str, base_url: Option<&str>) -> Result<Self> {
        let endpoint = derive_graphql_url(base_url);
        debug!(endpoint = %endpoint, "Creating GraphQL client");

        let http = Client::builder()
            .user_agent("greport")
            .build()
            .map_err(|e| Error::Custom(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            endpoint,
            token: token.to_string(),
        })
    }

    /// Execute a GraphQL query and deserialize the response.
    pub async fn query<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let request_body = GraphQLRequest { query, variables };

        let response = self
            .http
            .post(&self.endpoint)
            .header("Authorization", format!("bearer {}", self.token))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("GraphQL request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::GraphQL(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let gql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| Error::GraphQL(format!("Failed to parse GraphQL response: {}", e)))?;

        if let Some(errors) = gql_response.errors {
            if !errors.is_empty() {
                return Err(classify_graphql_errors(&errors));
            }
        }

        gql_response
            .data
            .ok_or_else(|| Error::GraphQL("GraphQL response contained no data".to_string()))
    }

    // -----------------------------------------------------------------------
    // High-level project operations
    // -----------------------------------------------------------------------

    /// List all projects for an organization.
    pub async fn list_org_projects(&self, org: &str) -> Result<Vec<Project>> {
        let mut all_projects = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "org": org,
                "first": 100,
                "after": cursor,
            });

            let data: OrgProjectsData = self.query(LIST_ORG_PROJECTS, variables).await?;
            let connection = data.organization.projects_v2;

            for node in connection.nodes {
                all_projects.push(convert_project_summary(node, org));
            }

            if connection.page_info.has_next_page {
                cursor = connection.page_info.end_cursor;
            } else {
                break;
            }
        }

        debug!(org = org, count = all_projects.len(), "Fetched projects");
        Ok(all_projects)
    }

    /// Get a single project with field definitions.
    pub async fn get_project_detail(
        &self,
        org: &str,
        project_number: u64,
    ) -> Result<Project> {
        let variables = serde_json::json!({
            "org": org,
            "number": project_number as i64,
        });

        let data: OrgProjectDetailData = self.query(GET_PROJECT_WITH_FIELDS, variables).await?;
        let gql_project = data.organization.project_v2.ok_or_else(|| {
            Error::NotFound(format!("Project #{} not found in org '{}'", project_number, org))
        })?;

        let fields = gql_project
            .fields
            .as_ref()
            .map(|f| f.nodes.iter().map(convert_field).collect())
            .unwrap_or_default();

        let mut project = convert_project_summary(
            GqlProject {
                id: gql_project.id,
                number: gql_project.number,
                title: gql_project.title,
                short_description: gql_project.short_description,
                url: gql_project.url,
                closed: gql_project.closed,
                created_at: gql_project.created_at,
                updated_at: gql_project.updated_at,
                items: gql_project.items,
            },
            org,
        );
        project.fields = fields;

        Ok(project)
    }

    /// List all items in a project (handles pagination).
    pub async fn list_items(&self, project_node_id: &str) -> Result<Vec<ProjectItem>> {
        let mut all_items = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "nodeId": project_node_id,
                "first": 100,
                "after": cursor,
            });

            let data: NodeItemsData = self.query(LIST_PROJECT_ITEMS, variables).await?;
            let project_data = data.node.ok_or_else(|| {
                Error::NotFound(format!("Project node '{}' not found", project_node_id))
            })?;

            let connection = project_data.items;

            for node in connection.nodes {
                if let Some(item) = convert_item(node) {
                    all_items.push(item);
                }
            }

            if connection.page_info.has_next_page {
                cursor = connection.page_info.end_cursor;
            } else {
                break;
            }
        }

        debug!(
            project = project_node_id,
            count = all_items.len(),
            "Fetched project items"
        );
        Ok(all_items)
    }
}

// ---------------------------------------------------------------------------
// Error classification
// ---------------------------------------------------------------------------

/// Classify GraphQL errors into typed Error variants.
fn classify_graphql_errors(errors: &[GraphQLError]) -> Error {
    for err in errors {
        let msg = &err.message;
        let msg_lower = msg.to_lowercase();

        // Permission errors
        if msg_lower.contains("resource not accessible by personal access token")
            || msg_lower.contains("read:project")
            || msg_lower.contains("insufficient scopes")
        {
            return Error::MissingProjectScope {
                org: "unknown".to_string(),
            };
        }

        // Not found
        if msg_lower.contains("could not resolve to") {
            return Error::NotFound(msg.clone());
        }

        // GHES version too old
        if msg_lower.contains("projectsv2") && msg_lower.contains("doesn't exist") {
            return Error::ProjectsNotAvailable;
        }
    }

    // Generic fallback
    let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
    Error::GraphQL(messages.join("; "))
}

// ---------------------------------------------------------------------------
// GraphQL response types (private serde structs)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct Connection<T> {
    nodes: Vec<T>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Clone, Deserialize)]
struct TotalCount {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

// -- List projects response --

#[derive(Deserialize)]
struct OrgProjectsData {
    organization: OrgProjectsOrg,
}

#[derive(Deserialize)]
struct OrgProjectsOrg {
    #[serde(rename = "projectsV2")]
    projects_v2: Connection<GqlProject>,
}

#[derive(Clone, Deserialize)]
struct GqlProject {
    id: String,
    number: u64,
    title: String,
    #[serde(rename = "shortDescription")]
    short_description: Option<String>,
    url: String,
    closed: bool,
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
    items: Option<TotalCount>,
}

// -- Get project detail response --

#[derive(Deserialize)]
struct OrgProjectDetailData {
    organization: OrgProjectDetailOrg,
}

#[derive(Deserialize)]
struct OrgProjectDetailOrg {
    #[serde(rename = "projectV2")]
    project_v2: Option<GqlProjectDetail>,
}

#[derive(Deserialize)]
struct GqlProjectDetail {
    id: String,
    number: u64,
    title: String,
    #[serde(rename = "shortDescription")]
    short_description: Option<String>,
    url: String,
    closed: bool,
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
    items: Option<TotalCount>,
    fields: Option<FieldsConnection>,
}

#[derive(Deserialize)]
struct FieldsConnection {
    nodes: Vec<GqlField>,
}

#[derive(Deserialize)]
struct GqlField {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "dataType")]
    data_type: Option<String>,
    options: Option<Vec<GqlSelectOption>>,
    configuration: Option<GqlIterationConfig>,
}

#[derive(Deserialize)]
struct GqlSelectOption {
    id: String,
    name: String,
    color: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct GqlIterationConfig {
    iterations: Vec<GqlIteration>,
}

#[derive(Deserialize)]
struct GqlIteration {
    id: String,
    title: String,
    #[serde(rename = "startDate")]
    start_date: String,
    duration: u32,
}

// -- List items response --

#[derive(Deserialize)]
struct NodeItemsData {
    node: Option<GqlProjectNode>,
}

#[derive(Deserialize)]
struct GqlProjectNode {
    items: Connection<GqlItem>,
}

#[derive(Deserialize)]
struct GqlItem {
    id: String,
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    updated_at: DateTime<Utc>,
    content: Option<GqlItemContent>,
    #[serde(rename = "fieldValues")]
    field_values: Option<GqlFieldValuesConnection>,
}

/// The content union -- GitHub returns different shapes based on type.
/// We use flatten + Options to handle the union.
#[derive(Deserialize)]
struct GqlItemContent {
    // Common to Issue and PR
    number: Option<u64>,
    title: Option<String>,
    state: Option<String>,
    url: Option<String>,
    repository: Option<GqlRepo>,
    // Issue-specific
    assignees: Option<GqlLoginNodes>,
    labels: Option<GqlLabelNodes>,
    // PR-specific
    merged: Option<bool>,
    author: Option<GqlLogin>,
    // DraftIssue-specific
    body: Option<String>,
}

#[derive(Deserialize)]
struct GqlRepo {
    #[serde(rename = "nameWithOwner")]
    name_with_owner: String,
}

#[derive(Deserialize)]
struct GqlLoginNodes {
    nodes: Vec<GqlLogin>,
}

#[derive(Deserialize)]
struct GqlLogin {
    login: Option<String>,
}

#[derive(Deserialize)]
struct GqlLabelNodes {
    nodes: Vec<GqlLabel>,
}

#[derive(Deserialize)]
struct GqlLabel {
    name: String,
    color: String,
}

#[derive(Deserialize)]
struct GqlFieldValuesConnection {
    nodes: Vec<GqlFieldValueNode>,
}

/// Field value node -- union type handled via Options.
#[derive(Deserialize)]
struct GqlFieldValueNode {
    // Text
    text: Option<String>,
    // Number
    number: Option<f64>,
    // Date
    date: Option<String>,
    // SingleSelect
    name: Option<String>,
    #[serde(rename = "optionId")]
    option_id: Option<String>,
    // Iteration
    title: Option<String>,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
    duration: Option<u32>,
    #[serde(rename = "iterationId")]
    iteration_id: Option<String>,
    // Common
    field: Option<GqlFieldRef>,
}

#[derive(Deserialize)]
struct GqlFieldRef {
    name: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversion functions (GraphQL response -> domain models)
// ---------------------------------------------------------------------------

fn convert_project_summary(gql: GqlProject, org: &str) -> Project {
    Project {
        node_id: gql.id,
        number: gql.number,
        title: gql.title,
        description: gql.short_description,
        url: gql.url,
        closed: gql.closed,
        owner: org.to_string(),
        created_at: gql.created_at,
        updated_at: gql.updated_at,
        fields: Vec::new(),
        total_items: gql.items.map(|i| i.total_count).unwrap_or(0),
    }
}

fn convert_field(gql: &GqlField) -> ProjectField {
    let node_id = gql.id.clone().unwrap_or_default();
    let name = gql.name.clone().unwrap_or_default();
    let data_type = gql.data_type.as_deref().unwrap_or("");

    let field_type = match data_type {
        "TEXT" => ProjectFieldType::Text,
        "NUMBER" => ProjectFieldType::Number,
        "DATE" => ProjectFieldType::Date,
        "SINGLE_SELECT" => {
            let options = gql
                .options
                .as_ref()
                .map(|opts| {
                    opts.iter()
                        .map(|o| SelectOption {
                            id: o.id.clone(),
                            name: o.name.clone(),
                            color: o.color.clone(),
                            description: o.description.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            ProjectFieldType::SingleSelect { options }
        }
        "ITERATION" => {
            let iterations = gql
                .configuration
                .as_ref()
                .map(|cfg| {
                    cfg.iterations
                        .iter()
                        .map(|it| IterationValue {
                            id: it.id.clone(),
                            title: it.title.clone(),
                            start_date: it.start_date.clone(),
                            duration: it.duration,
                        })
                        .collect()
                })
                .unwrap_or_default();
            ProjectFieldType::Iteration { iterations }
        }
        "TITLE" | "ASSIGNEES" | "LABELS" | "MILESTONE" | "REPOSITORY" | "REVIEWERS"
        | "LINKED_PULL_REQUESTS" | "TRACKS" | "TRACKED_BY" => ProjectFieldType::BuiltIn,
        _ => {
            warn!(data_type = data_type, name = %name, "Unknown field type, treating as BuiltIn");
            ProjectFieldType::BuiltIn
        }
    };

    ProjectField {
        node_id,
        name,
        field_type,
    }
}

fn convert_item(gql: GqlItem) -> Option<ProjectItem> {
    let content = match gql.content {
        Some(c) => convert_content(c)?,
        None => return None,
    };

    let field_values = gql
        .field_values
        .map(|fv| {
            fv.nodes
                .into_iter()
                .filter_map(convert_field_value)
                .collect()
        })
        .unwrap_or_default();

    Some(ProjectItem {
        node_id: gql.id,
        content,
        field_values,
        created_at: gql.created_at,
        updated_at: gql.updated_at,
    })
}

fn convert_content(gql: GqlItemContent) -> Option<ProjectItemContent> {
    // Determine content type by which fields are present
    if let Some(repo) = &gql.repository {
        // Has a repository -> Issue or PR
        if gql.merged.is_some() {
            // PR (has merged field)
            Some(ProjectItemContent::PullRequest {
                number: gql.number.unwrap_or(0),
                title: gql.title.unwrap_or_default(),
                state: gql.state.unwrap_or_default(),
                url: gql.url.unwrap_or_default(),
                repository: repo.name_with_owner.clone(),
                merged: gql.merged.unwrap_or(false),
                author: gql
                    .author
                    .and_then(|a| a.login)
                    .unwrap_or_else(|| "unknown".to_string()),
            })
        } else {
            // Issue
            let assignees = gql
                .assignees
                .map(|a| a.nodes.into_iter().filter_map(|n| n.login).collect())
                .unwrap_or_default();
            let labels = gql
                .labels
                .map(|l| {
                    l.nodes
                        .into_iter()
                        .map(|n| LabelInfo {
                            name: n.name,
                            color: n.color,
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(ProjectItemContent::Issue {
                number: gql.number.unwrap_or(0),
                title: gql.title.unwrap_or_default(),
                state: gql.state.unwrap_or_default(),
                url: gql.url.unwrap_or_default(),
                repository: repo.name_with_owner.clone(),
                assignees,
                labels,
            })
        }
    } else if gql.title.is_some() {
        // No repository -> DraftIssue
        let assignees = gql
            .assignees
            .map(|a| a.nodes.into_iter().filter_map(|n| n.login).collect())
            .unwrap_or_default();

        Some(ProjectItemContent::DraftIssue {
            title: gql.title.unwrap_or_default(),
            body: gql.body,
            assignees,
        })
    } else {
        // Empty or redacted content
        None
    }
}

fn convert_field_value(gql: GqlFieldValueNode) -> Option<ProjectFieldValue> {
    let field_name = gql.field.as_ref()?.name.as_ref()?.clone();

    // Determine value type by which fields are populated.
    // Order matters: check iteration first (has title + startDate + duration),
    // then single-select (has name + optionId), then simpler types.

    let value = if gql.iteration_id.is_some() {
        FieldValue::Iteration {
            title: gql.title.unwrap_or_default(),
            start_date: gql.start_date.unwrap_or_default(),
            duration: gql.duration.unwrap_or(0),
            iteration_id: gql.iteration_id.unwrap_or_default(),
        }
    } else if gql.option_id.is_some() {
        FieldValue::SingleSelect {
            name: gql.name.unwrap_or_default(),
            option_id: gql.option_id.unwrap_or_default(),
        }
    } else if let Some(text) = gql.text {
        FieldValue::Text { value: text }
    } else if let Some(num) = gql.number {
        FieldValue::Number { value: num }
    } else if let Some(date) = gql.date {
        FieldValue::Date { value: date }
    } else {
        // Empty or unrecognized field value type
        return None;
    };

    Some(ProjectFieldValue { field_name, value })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- derive_graphql_url tests --

    #[test]
    fn test_derive_graphql_url_github_com() {
        assert_eq!(
            derive_graphql_url(None),
            "https://api.github.com/graphql"
        );
    }

    #[test]
    fn test_derive_graphql_url_ghe_api_v3() {
        assert_eq!(
            derive_graphql_url(Some("https://ghe.corp.com/api/v3")),
            "https://ghe.corp.com/api/graphql"
        );
    }

    #[test]
    fn test_derive_graphql_url_ghe_api_v3_trailing_slash() {
        assert_eq!(
            derive_graphql_url(Some("https://ghe.corp.com/api/v3/")),
            "https://ghe.corp.com/api/graphql"
        );
    }

    #[test]
    fn test_derive_graphql_url_ghe_api() {
        assert_eq!(
            derive_graphql_url(Some("https://ghe.corp.com/api")),
            "https://ghe.corp.com/api/graphql"
        );
    }

    #[test]
    fn test_derive_graphql_url_bare_hostname() {
        assert_eq!(
            derive_graphql_url(Some("https://ghe.corp.com")),
            "https://ghe.corp.com/api/graphql"
        );
    }

    // -- Response parsing tests --

    #[test]
    fn test_parse_projects_response() {
        let json = r#"{
            "data": {
                "organization": {
                    "projectsV2": {
                        "nodes": [
                            {
                                "id": "PVT_kwDOABC",
                                "number": 5,
                                "title": "Sprint Q1",
                                "shortDescription": "Q1 planning",
                                "url": "https://github.com/orgs/my-org/projects/5",
                                "closed": false,
                                "createdAt": "2026-01-15T10:00:00Z",
                                "updatedAt": "2026-02-10T14:30:00Z",
                                "items": { "totalCount": 47 }
                            },
                            {
                                "id": "PVT_kwDODEF",
                                "number": 4,
                                "title": "Infra Upgrades",
                                "shortDescription": null,
                                "url": "https://github.com/orgs/my-org/projects/4",
                                "closed": true,
                                "createdAt": "2025-11-01T08:00:00Z",
                                "updatedAt": "2026-01-30T12:00:00Z",
                                "items": { "totalCount": 23 }
                            }
                        ],
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        }
                    }
                }
            }
        }"#;

        let resp: GraphQLResponse<OrgProjectsData> = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();
        let nodes = &data.organization.projects_v2.nodes;

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].number, 5);
        assert_eq!(nodes[0].title, "Sprint Q1");
        assert!(!nodes[0].closed);
        assert_eq!(nodes[0].items.as_ref().unwrap().total_count, 47);
        assert_eq!(nodes[1].number, 4);
        assert!(nodes[1].closed);

        // Test conversion
        let project = convert_project_summary(nodes[0].clone(), "my-org");
        assert_eq!(project.owner, "my-org");
        assert_eq!(project.total_items, 47);
        assert_eq!(project.node_id, "PVT_kwDOABC");
    }

    #[test]
    fn test_parse_project_with_fields() {
        let json = r#"{
            "data": {
                "organization": {
                    "projectV2": {
                        "id": "PVT_kwDOABC",
                        "number": 5,
                        "title": "Sprint Q1",
                        "shortDescription": null,
                        "url": "https://github.com/orgs/my-org/projects/5",
                        "closed": false,
                        "createdAt": "2026-01-15T10:00:00Z",
                        "updatedAt": "2026-02-10T14:30:00Z",
                        "items": { "totalCount": 47 },
                        "fields": {
                            "nodes": [
                                {
                                    "id": "PVTF_title",
                                    "name": "Title",
                                    "dataType": "TITLE"
                                },
                                {
                                    "id": "PVTSSF_status",
                                    "name": "Status",
                                    "dataType": "SINGLE_SELECT",
                                    "options": [
                                        { "id": "opt1", "name": "Todo", "color": "GRAY", "description": null },
                                        { "id": "opt2", "name": "In Progress", "color": "YELLOW", "description": null },
                                        { "id": "opt3", "name": "Done", "color": "GREEN", "description": null }
                                    ]
                                },
                                {
                                    "id": "PVTIF_sprint",
                                    "name": "Sprint",
                                    "dataType": "ITERATION",
                                    "configuration": {
                                        "iterations": [
                                            { "id": "it1", "title": "Sprint 1", "startDate": "2026-01-06", "duration": 14 },
                                            { "id": "it2", "title": "Sprint 2", "startDate": "2026-01-20", "duration": 14 }
                                        ]
                                    }
                                },
                                {
                                    "id": "PVTF_pts",
                                    "name": "Story Points",
                                    "dataType": "NUMBER"
                                }
                            ]
                        }
                    }
                }
            }
        }"#;

        let resp: GraphQLResponse<OrgProjectDetailData> = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();
        let project = data.organization.project_v2.unwrap();
        let fields: Vec<ProjectField> = project
            .fields
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .map(convert_field)
            .collect();

        assert_eq!(fields.len(), 4);

        // Title -> BuiltIn
        assert_eq!(fields[0].name, "Title");
        assert!(matches!(fields[0].field_type, ProjectFieldType::BuiltIn));

        // Status -> SingleSelect with 3 options
        assert_eq!(fields[1].name, "Status");
        match &fields[1].field_type {
            ProjectFieldType::SingleSelect { options } => {
                assert_eq!(options.len(), 3);
                assert_eq!(options[0].name, "Todo");
                assert_eq!(options[1].name, "In Progress");
                assert_eq!(options[2].name, "Done");
            }
            other => panic!("Expected SingleSelect, got {:?}", other),
        }

        // Sprint -> Iteration with 2 iterations
        assert_eq!(fields[2].name, "Sprint");
        match &fields[2].field_type {
            ProjectFieldType::Iteration { iterations } => {
                assert_eq!(iterations.len(), 2);
                assert_eq!(iterations[0].title, "Sprint 1");
                assert_eq!(iterations[0].duration, 14);
                assert_eq!(iterations[1].title, "Sprint 2");
            }
            other => panic!("Expected Iteration, got {:?}", other),
        }

        // Story Points -> Number
        assert_eq!(fields[3].name, "Story Points");
        assert!(matches!(fields[3].field_type, ProjectFieldType::Number));
    }

    #[test]
    fn test_parse_project_items() {
        let json = r#"{
            "data": {
                "node": {
                    "items": {
                        "nodes": [
                            {
                                "id": "PVTI_issue",
                                "createdAt": "2026-01-20T09:00:00Z",
                                "updatedAt": "2026-02-08T16:00:00Z",
                                "content": {
                                    "number": 42,
                                    "title": "Implement auth",
                                    "state": "OPEN",
                                    "url": "https://github.com/my-org/repo/issues/42",
                                    "repository": { "nameWithOwner": "my-org/repo" },
                                    "assignees": { "nodes": [{ "login": "alice" }, { "login": "bob" }] },
                                    "labels": { "nodes": [{ "name": "feature", "color": "0e8a16" }] }
                                },
                                "fieldValues": {
                                    "nodes": [
                                        {
                                            "name": "In Progress",
                                            "optionId": "opt2",
                                            "field": { "name": "Status" }
                                        },
                                        {
                                            "number": 5.0,
                                            "field": { "name": "Story Points" }
                                        }
                                    ]
                                }
                            },
                            {
                                "id": "PVTI_pr",
                                "createdAt": "2026-01-22T10:00:00Z",
                                "updatedAt": "2026-02-09T11:00:00Z",
                                "content": {
                                    "number": 100,
                                    "title": "Fix login bug",
                                    "state": "MERGED",
                                    "url": "https://github.com/my-org/repo/pull/100",
                                    "merged": true,
                                    "repository": { "nameWithOwner": "my-org/repo" },
                                    "author": { "login": "charlie" }
                                },
                                "fieldValues": {
                                    "nodes": [
                                        {
                                            "name": "Done",
                                            "optionId": "opt3",
                                            "field": { "name": "Status" }
                                        }
                                    ]
                                }
                            },
                            {
                                "id": "PVTI_draft",
                                "createdAt": "2026-02-01T08:00:00Z",
                                "updatedAt": "2026-02-01T08:00:00Z",
                                "content": {
                                    "title": "Research caching",
                                    "body": "Investigate Redis vs Memcached",
                                    "assignees": { "nodes": [{ "login": "alice" }] }
                                },
                                "fieldValues": {
                                    "nodes": []
                                }
                            }
                        ],
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        }
                    }
                }
            }
        }"#;

        let resp: GraphQLResponse<NodeItemsData> = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();
        let items_conn = data.node.unwrap().items;
        let items: Vec<ProjectItem> = items_conn
            .nodes
            .into_iter()
            .filter_map(convert_item)
            .collect();

        assert_eq!(items.len(), 3);

        // First item: Issue
        match &items[0].content {
            ProjectItemContent::Issue {
                number,
                title,
                state,
                repository,
                assignees,
                labels,
                ..
            } => {
                assert_eq!(*number, 42);
                assert_eq!(title, "Implement auth");
                assert_eq!(state, "OPEN");
                assert_eq!(repository, "my-org/repo");
                assert_eq!(assignees, &["alice", "bob"]);
                assert_eq!(labels.len(), 1);
                assert_eq!(labels[0].name, "feature");
            }
            other => panic!("Expected Issue, got {:?}", other),
        }
        assert_eq!(items[0].field_values.len(), 2);
        assert_eq!(items[0].field_values[0].field_name, "Status");
        match &items[0].field_values[0].value {
            FieldValue::SingleSelect { name, .. } => assert_eq!(name, "In Progress"),
            other => panic!("Expected SingleSelect, got {:?}", other),
        }
        match &items[0].field_values[1].value {
            FieldValue::Number { value } => assert!((value - 5.0).abs() < f64::EPSILON),
            other => panic!("Expected Number, got {:?}", other),
        }

        // Second item: PR
        match &items[1].content {
            ProjectItemContent::PullRequest {
                number,
                title,
                merged,
                author,
                ..
            } => {
                assert_eq!(*number, 100);
                assert_eq!(title, "Fix login bug");
                assert!(*merged);
                assert_eq!(author, "charlie");
            }
            other => panic!("Expected PullRequest, got {:?}", other),
        }

        // Third item: DraftIssue
        match &items[2].content {
            ProjectItemContent::DraftIssue {
                title,
                body,
                assignees,
            } => {
                assert_eq!(title, "Research caching");
                assert_eq!(body.as_deref(), Some("Investigate Redis vs Memcached"));
                assert_eq!(assignees, &["alice"]);
            }
            other => panic!("Expected DraftIssue, got {:?}", other),
        }
        assert!(items[2].field_values.is_empty());
    }

    // -- Error classification tests --

    #[test]
    fn test_classify_permission_error() {
        let errors = vec![GraphQLError {
            message: "Resource not accessible by personal access token".to_string(),
            error_type: Some("FORBIDDEN".to_string()),
        }];

        let err = classify_graphql_errors(&errors);
        assert!(matches!(err, Error::MissingProjectScope { .. }));
    }

    #[test]
    fn test_classify_not_found_error() {
        let errors = vec![GraphQLError {
            message: "Could not resolve to an Organization with the login of 'nonexistent'".to_string(),
            error_type: Some("NOT_FOUND".to_string()),
        }];

        let err = classify_graphql_errors(&errors);
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[test]
    fn test_classify_generic_error() {
        let errors = vec![
            GraphQLError {
                message: "Something went wrong".to_string(),
                error_type: None,
            },
            GraphQLError {
                message: "Another error".to_string(),
                error_type: None,
            },
        ];

        let err = classify_graphql_errors(&errors);
        match err {
            Error::GraphQL(msg) => {
                assert!(msg.contains("Something went wrong"));
                assert!(msg.contains("Another error"));
            }
            other => panic!("Expected GraphQL error, got {:?}", other),
        }
    }

    // -- Field value conversion tests --

    #[test]
    fn test_convert_iteration_field_value() {
        let node = GqlFieldValueNode {
            text: None,
            number: None,
            date: None,
            name: None,
            option_id: None,
            title: Some("Sprint 1".to_string()),
            start_date: Some("2026-01-06".to_string()),
            duration: Some(14),
            iteration_id: Some("it1".to_string()),
            field: Some(GqlFieldRef {
                name: Some("Sprint".to_string()),
            }),
        };

        let fv = convert_field_value(node).unwrap();
        assert_eq!(fv.field_name, "Sprint");
        match fv.value {
            FieldValue::Iteration {
                title,
                start_date,
                duration,
                iteration_id,
            } => {
                assert_eq!(title, "Sprint 1");
                assert_eq!(start_date, "2026-01-06");
                assert_eq!(duration, 14);
                assert_eq!(iteration_id, "it1");
            }
            other => panic!("Expected Iteration, got {:?}", other),
        }
    }

    #[test]
    fn test_convert_date_field_value() {
        let node = GqlFieldValueNode {
            text: None,
            number: None,
            date: Some("2026-02-28".to_string()),
            name: None,
            option_id: None,
            title: None,
            start_date: None,
            duration: None,
            iteration_id: None,
            field: Some(GqlFieldRef {
                name: Some("Due Date".to_string()),
            }),
        };

        let fv = convert_field_value(node).unwrap();
        assert_eq!(fv.field_name, "Due Date");
        match fv.value {
            FieldValue::Date { value } => assert_eq!(value, "2026-02-28"),
            other => panic!("Expected Date, got {:?}", other),
        }
    }

    #[test]
    fn test_convert_field_value_no_field_ref() {
        let node = GqlFieldValueNode {
            text: Some("hello".to_string()),
            number: None,
            date: None,
            name: None,
            option_id: None,
            title: None,
            start_date: None,
            duration: None,
            iteration_id: None,
            field: None, // No field reference
        };

        assert!(convert_field_value(node).is_none());
    }
}
