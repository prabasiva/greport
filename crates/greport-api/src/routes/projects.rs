//! GitHub Projects V2 route handlers

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ApiError;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::state::AppState;
use greport_db::models::{ProjectFieldRow, ProjectItemRow, ProjectRow};

// =============================================================================
// Response types
// =============================================================================

#[derive(Serialize)]
pub struct ProjectSummary {
    pub number: i64,
    pub owner: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub closed: bool,
    pub total_items: i32,
    pub synced_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ProjectDetail {
    pub number: i64,
    pub owner: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub closed: bool,
    pub total_items: i32,
    pub synced_at: DateTime<Utc>,
    pub fields: Vec<ProjectFieldSummary>,
}

#[derive(Serialize)]
pub struct ProjectFieldSummary {
    pub name: String,
    pub field_type: String,
    pub config_json: Option<Value>,
}

#[derive(Serialize)]
pub struct ProjectItemResponse {
    pub node_id: String,
    pub content_type: String,
    pub content_number: Option<i64>,
    pub content_title: String,
    pub content_state: Option<String>,
    pub content_url: Option<String>,
    pub content_repository: Option<String>,
    pub field_values: Option<Value>,
}

#[derive(Serialize)]
pub struct ProjectMetrics {
    pub project_number: i64,
    pub project_title: String,
    pub total_items: i32,
    pub by_status: Vec<StatusCount>,
    pub by_content_type: Vec<ContentTypeCount>,
}

#[derive(Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct ContentTypeCount {
    pub content_type: String,
    pub count: i64,
}

// =============================================================================
// Query parameter structs
// =============================================================================

#[derive(Deserialize)]
pub struct ListProjectsQuery {
    pub include_closed: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListItemsQuery {
    pub content_type: Option<String>,
    pub state: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// =============================================================================
// Endpoint handlers
// =============================================================================

/// GET /api/v1/orgs/{org}/projects
pub async fn list_projects(
    State(state): State<AppState>,
    Path(org): Path<String>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ApiResponse<Vec<ProjectSummary>>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for projects".into()))?;

    let include_closed = query.include_closed.unwrap_or(false);
    let rows = greport_db::queries::list_projects(pool, &org, include_closed).await?;
    let summaries: Vec<ProjectSummary> = rows.iter().map(project_row_to_summary).collect();

    Ok(Json(ApiResponse::ok(summaries)))
}

/// GET /api/v1/orgs/{org}/projects/{number}
pub async fn get_project(
    State(state): State<AppState>,
    Path((org, number)): Path<(String, i64)>,
) -> Result<Json<ApiResponse<ProjectDetail>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for projects".into()))?;

    let project = greport_db::queries::get_project(pool, &org, number)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Project {} not found for org {}", number, org))
        })?;

    let field_rows = greport_db::queries::list_project_fields(pool, &project.node_id).await?;
    let fields: Vec<ProjectFieldSummary> = field_rows.iter().map(field_row_to_summary).collect();

    let detail = ProjectDetail {
        number: project.number,
        owner: project.owner,
        title: project.title,
        description: project.description,
        url: project.url,
        closed: project.closed,
        total_items: project.total_items,
        synced_at: project.synced_at,
        fields,
    };

    Ok(Json(ApiResponse::ok(detail)))
}

/// GET /api/v1/orgs/{org}/projects/{number}/items
pub async fn list_project_items(
    State(state): State<AppState>,
    Path((org, number)): Path<(String, i64)>,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<PaginatedResponse<ProjectItemResponse>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for projects".into()))?;

    let project = greport_db::queries::get_project(pool, &org, number)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Project {} not found for org {}", number, org))
        })?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(30).min(100);
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;

    // Get total count (unfiltered by pagination)
    let all_items = greport_db::queries::list_project_items(
        pool,
        &project.node_id,
        query.content_type.as_deref(),
        query.state.as_deref(),
        None,
        None,
    )
    .await?;
    let total = all_items.len() as u32;

    // Get paginated items
    let items = greport_db::queries::list_project_items(
        pool,
        &project.node_id,
        query.content_type.as_deref(),
        query.state.as_deref(),
        Some(limit),
        Some(offset),
    )
    .await?;

    let response_items: Vec<ProjectItemResponse> = items.iter().map(item_row_to_response).collect();

    Ok(Json(PaginatedResponse::new(
        response_items,
        page,
        per_page,
        total,
    )))
}

/// GET /api/v1/orgs/{org}/projects/{number}/metrics
pub async fn get_project_metrics(
    State(state): State<AppState>,
    Path((org, number)): Path<(String, i64)>,
) -> Result<Json<ApiResponse<ProjectMetrics>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for projects".into()))?;

    let project = greport_db::queries::get_project(pool, &org, number)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("Project {} not found for org {}", number, org))
        })?;

    // Status breakdown
    let status_rows =
        greport_db::queries::count_project_items_by_status(pool, &project.node_id, "Status")
            .await?;
    let by_status: Vec<StatusCount> = status_rows
        .into_iter()
        .map(|(status, count)| StatusCount { status, count })
        .collect();

    // Content type breakdown
    let all_items =
        greport_db::queries::list_project_items(pool, &project.node_id, None, None, None, None)
            .await?;

    let mut type_counts: HashMap<String, i64> = HashMap::new();
    for item in &all_items {
        *type_counts.entry(item.content_type.clone()).or_insert(0) += 1;
    }
    let mut by_content_type: Vec<ContentTypeCount> = type_counts
        .into_iter()
        .map(|(content_type, count)| ContentTypeCount {
            content_type,
            count,
        })
        .collect();
    by_content_type.sort_by(|a, b| b.count.cmp(&a.count));

    let metrics = ProjectMetrics {
        project_number: project.number,
        project_title: project.title,
        total_items: project.total_items,
        by_status,
        by_content_type,
    };

    Ok(Json(ApiResponse::ok(metrics)))
}

/// GET /api/v1/aggregate/projects
pub async fn aggregate_projects(
    State(state): State<AppState>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ApiResponse<Vec<ProjectSummary>>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate projects".into()))?;

    let include_closed = query.include_closed.unwrap_or(false);
    let mut all_summaries: Vec<ProjectSummary> = Vec::new();

    for entry in state.registry.org_entries() {
        let rows = greport_db::queries::list_projects(pool, &entry.name, include_closed).await?;
        all_summaries.extend(rows.iter().map(project_row_to_summary));
    }

    Ok(Json(ApiResponse::ok(all_summaries)))
}

// =============================================================================
// Conversion helpers
// =============================================================================

fn project_row_to_summary(row: &ProjectRow) -> ProjectSummary {
    ProjectSummary {
        number: row.number,
        owner: row.owner.clone(),
        title: row.title.clone(),
        description: row.description.clone(),
        url: row.url.clone(),
        closed: row.closed,
        total_items: row.total_items,
        synced_at: row.synced_at,
    }
}

fn field_row_to_summary(row: &ProjectFieldRow) -> ProjectFieldSummary {
    ProjectFieldSummary {
        name: row.name.clone(),
        field_type: row.field_type.clone(),
        config_json: row.config_json.clone(),
    }
}

fn item_row_to_response(row: &ProjectItemRow) -> ProjectItemResponse {
    ProjectItemResponse {
        node_id: row.node_id.clone(),
        content_type: row.content_type.clone(),
        content_number: row.content_number,
        content_title: row.content_title.clone(),
        content_state: row.content_state.clone(),
        content_url: row.content_url.clone(),
        content_repository: row.content_repository.clone(),
        field_values: row.field_values_json.clone(),
    }
}
