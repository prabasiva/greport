//! GitHub Projects V2 data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A GitHub Project (V2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// GitHub GraphQL node ID (opaque string like "PVT_kwDO...")
    pub node_id: String,
    /// Project number within the organization
    pub number: u64,
    /// Project title
    pub title: String,
    /// Short description
    pub description: Option<String>,
    /// Web URL for the project
    pub url: String,
    /// Whether the project is closed
    pub closed: bool,
    /// Organization or user that owns the project
    pub owner: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Custom field definitions
    pub fields: Vec<ProjectField>,
    /// Total item count
    pub total_items: u32,
}

/// A field definition on a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectField {
    /// GraphQL node ID
    pub node_id: String,
    /// Display name (e.g., "Status", "Priority", "Sprint")
    pub name: String,
    /// Field type with embedded configuration
    pub field_type: ProjectFieldType,
}

/// Discriminated field type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectFieldType {
    /// Plain text field
    Text,
    /// Numeric field
    Number,
    /// Date field
    Date,
    /// Single-select with predefined options
    SingleSelect { options: Vec<SelectOption> },
    /// Iteration/sprint field
    Iteration { iterations: Vec<IterationValue> },
    /// Built-in fields (Title, Assignees, Labels, etc.)
    BuiltIn,
}

/// A single-select option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    /// Option ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Color code
    pub color: Option<String>,
    /// Description text
    pub description: Option<String>,
}

/// An iteration/sprint value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationValue {
    /// Iteration ID
    pub id: String,
    /// Display title (e.g., "Sprint 1")
    pub title: String,
    /// Start date (YYYY-MM-DD)
    pub start_date: String,
    /// Duration in days
    pub duration: u32,
}

/// An item on a project board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItem {
    /// GraphQL node ID
    pub node_id: String,
    /// The linked content (Issue, PR, or DraftIssue)
    pub content: ProjectItemContent,
    /// Field values for this item
    pub field_values: Vec<ProjectFieldValue>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// The content linked to a project item (union type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectItemContent {
    /// Linked issue
    Issue {
        number: u64,
        title: String,
        state: String,
        url: String,
        repository: String,
        assignees: Vec<String>,
        labels: Vec<LabelInfo>,
    },
    /// Linked pull request
    PullRequest {
        number: u64,
        title: String,
        state: String,
        url: String,
        repository: String,
        merged: bool,
        author: String,
    },
    /// Draft issue (not yet converted to a real issue)
    DraftIssue {
        title: String,
        body: Option<String>,
        assignees: Vec<String>,
    },
}

/// Label summary for project items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelInfo {
    /// Label name
    pub name: String,
    /// Color hex code
    pub color: String,
}

/// A field value on a project item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFieldValue {
    /// The field name this value belongs to
    pub field_name: String,
    /// The typed value
    pub value: FieldValue,
}

/// Typed field value (mirrors GraphQL union).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldValue {
    /// Text value
    Text { value: String },
    /// Numeric value
    Number { value: f64 },
    /// Date value (YYYY-MM-DD)
    Date { value: String },
    /// Single-select value
    SingleSelect { name: String, option_id: String },
    /// Iteration value
    Iteration {
        title: String,
        start_date: String,
        duration: u32,
        iteration_id: String,
    },
    /// No value set
    Empty,
}
