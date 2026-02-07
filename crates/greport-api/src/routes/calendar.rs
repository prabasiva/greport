//! Calendar route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use crate::convert;
use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use greport_core::models::{
    CalendarData, CalendarEvent, CalendarEventType, CalendarSummary, Issue, IssueState, Milestone,
    MilestoneState, PullRequest, Release,
};

#[derive(Deserialize)]
pub struct CalendarQuery {
    start_date: Option<String>,
    end_date: Option<String>,
    types: Option<String>,
}

/// Parse a date string (YYYY-MM-DD) into NaiveDate
fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Compute default date range: first of previous month to last of next month
fn default_date_range() -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    let year = today.year();
    let month = today.month();

    // Previous month
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    let start = NaiveDate::from_ymd_opt(prev_year, prev_month, 1).unwrap_or(today);

    // Next month last day
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    // Last day of next month = first day of month after next - 1
    let (after_year, after_month) = if next_month == 12 {
        (next_year + 1, 1)
    } else {
        (next_year, next_month + 1)
    };
    let end = NaiveDate::from_ymd_opt(after_year, after_month, 1)
        .unwrap_or(today)
        .pred_opt()
        .unwrap_or(today);

    (start, end)
}

/// Parse which event types the user wants
fn parse_types(types_str: Option<&str>) -> Vec<String> {
    types_str
        .unwrap_or("issues,milestones,releases,pulls")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect()
}

/// Build CalendarEvent objects from domain models
#[allow(clippy::too_many_arguments)]
fn build_calendar_events(
    issues: &[Issue],
    milestones: &[Milestone],
    releases: &[Release],
    pulls: &[PullRequest],
    repo_name: &str,
    types: &[String],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<CalendarEvent> {
    let mut events = Vec::new();

    let in_range = |dt: &chrono::DateTime<Utc>| -> bool {
        let d = dt.date_naive();
        d >= start && d <= end
    };

    if types.iter().any(|t| t == "issues") {
        for issue in issues {
            // Issue created
            if in_range(&issue.created_at) {
                events.push(CalendarEvent {
                    id: format!("{}-issue-created-{}", repo_name, issue.number),
                    event_type: CalendarEventType::IssueCreated,
                    title: issue.title.clone(),
                    date: issue.created_at,
                    number: Some(issue.number),
                    state: Some(match issue.state {
                        IssueState::Open => "open".to_string(),
                        IssueState::Closed => "closed".to_string(),
                    }),
                    repository: repo_name.to_string(),
                    labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                    milestone: issue.milestone.as_ref().map(|m| m.title.clone()),
                    url: format!("https://github.com/{}/issues/{}", repo_name, issue.number),
                });
            }

            // Issue closed
            if let Some(closed_at) = issue.closed_at {
                if in_range(&closed_at) {
                    events.push(CalendarEvent {
                        id: format!("{}-issue-closed-{}", repo_name, issue.number),
                        event_type: CalendarEventType::IssueClosed,
                        title: issue.title.clone(),
                        date: closed_at,
                        number: Some(issue.number),
                        state: Some("closed".to_string()),
                        repository: repo_name.to_string(),
                        labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                        milestone: issue.milestone.as_ref().map(|m| m.title.clone()),
                        url: format!("https://github.com/{}/issues/{}", repo_name, issue.number),
                    });
                }
            }
        }
    }

    if types.iter().any(|t| t == "milestones") {
        for ms in milestones {
            // Milestone due date
            if let Some(due_on) = ms.due_on {
                if in_range(&due_on) {
                    events.push(CalendarEvent {
                        id: format!("{}-milestone-due-{}", repo_name, ms.number),
                        event_type: CalendarEventType::MilestoneDue,
                        title: ms.title.clone(),
                        date: due_on,
                        number: Some(ms.number),
                        state: Some(match ms.state {
                            MilestoneState::Open => "open".to_string(),
                            MilestoneState::Closed => "closed".to_string(),
                        }),
                        repository: repo_name.to_string(),
                        labels: vec![],
                        milestone: Some(ms.title.clone()),
                        url: format!("https://github.com/{}/milestone/{}", repo_name, ms.number),
                    });
                }
            }

            // Milestone closed
            if let Some(closed_at) = ms.closed_at {
                if in_range(&closed_at) {
                    events.push(CalendarEvent {
                        id: format!("{}-milestone-closed-{}", repo_name, ms.number),
                        event_type: CalendarEventType::MilestoneClosed,
                        title: ms.title.clone(),
                        date: closed_at,
                        number: Some(ms.number),
                        state: Some("closed".to_string()),
                        repository: repo_name.to_string(),
                        labels: vec![],
                        milestone: Some(ms.title.clone()),
                        url: format!("https://github.com/{}/milestone/{}", repo_name, ms.number),
                    });
                }
            }
        }
    }

    if types.iter().any(|t| t == "releases") {
        for release in releases {
            if let Some(published_at) = release.published_at {
                if in_range(&published_at) {
                    events.push(CalendarEvent {
                        id: format!("{}-release-{}", repo_name, release.tag_name),
                        event_type: CalendarEventType::ReleasePublished,
                        title: release
                            .name
                            .clone()
                            .unwrap_or_else(|| release.tag_name.clone()),
                        date: published_at,
                        number: None,
                        state: None,
                        repository: repo_name.to_string(),
                        labels: vec![],
                        milestone: None,
                        url: format!(
                            "https://github.com/{}/releases/tag/{}",
                            repo_name, release.tag_name
                        ),
                    });
                }
            }
        }
    }

    if types.iter().any(|t| t == "pulls") {
        for pr in pulls {
            if let Some(merged_at) = pr.merged_at {
                if in_range(&merged_at) {
                    events.push(CalendarEvent {
                        id: format!("{}-pr-merged-{}", repo_name, pr.number),
                        event_type: CalendarEventType::PrMerged,
                        title: pr.title.clone(),
                        date: merged_at,
                        number: Some(pr.number),
                        state: Some("merged".to_string()),
                        repository: repo_name.to_string(),
                        labels: vec![],
                        milestone: None,
                        url: format!("https://github.com/{}/pull/{}", repo_name, pr.number),
                    });
                }
            }
        }
    }

    events
}

/// Compute summary from events
fn compute_summary(events: &[CalendarEvent]) -> CalendarSummary {
    let mut by_type: HashMap<String, usize> = HashMap::new();
    for event in events {
        let key = serde_json::to_value(event.event_type)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", event.event_type));
        *by_type.entry(key).or_insert(0) += 1;
    }
    CalendarSummary {
        total_events: events.len(),
        by_type,
    }
}

/// GET /api/v1/repos/{owner}/{repo}/calendar
pub async fn get_calendar(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<ApiResponse<CalendarData>>, ApiError> {
    let (default_start, default_end) = default_date_range();
    let start = query
        .start_date
        .as_deref()
        .and_then(parse_date)
        .unwrap_or(default_start);
    let end = query
        .end_date
        .as_deref()
        .and_then(parse_date)
        .unwrap_or(default_end);
    let types = parse_types(query.types.as_deref());

    let repo_name = format!("{}/{}", owner, repo);

    let start_dt = Utc.from_utc_datetime(&start.and_hms_opt(0, 0, 0).unwrap());
    let end_dt = Utc.from_utc_datetime(&end.and_hms_opt(23, 59, 59).unwrap());

    // DB-first path
    if let Some(pool) = &state.db {
        if let Some(repo_db_id) = convert::get_repo_db_id(pool, &owner, &repo).await {
            let mut issues = Vec::new();
            let mut milestones = Vec::new();
            let mut releases = Vec::new();
            let mut pulls = Vec::new();

            if types.iter().any(|t| t == "issues")
                && convert::has_synced_data(pool, repo_db_id, "issues").await
            {
                let rows = greport_db::queries::list_issues_in_date_range(
                    pool, repo_db_id, start_dt, end_dt,
                )
                .await?;
                for row in rows {
                    // Convert issue rows to domain Issue - use simplified conversion
                    let label_rows = greport_db::queries::get_issue_labels(pool, row.id).await?;
                    let labels: Vec<greport_core::models::Label> = label_rows
                        .into_iter()
                        .map(|l| greport_core::models::Label {
                            id: l.label_id,
                            name: l.label_name,
                            color: l.label_color.unwrap_or_default(),
                            description: None,
                        })
                        .collect();

                    let ms =
                        match row.milestone_id {
                            Some(ms_id) => greport_db::queries::get_milestone(pool, ms_id)
                                .await?
                                .map(|m| Milestone {
                                    id: m.id,
                                    number: m.number as u64,
                                    title: m.title,
                                    description: m.description,
                                    state: match m.state.as_str() {
                                        "closed" => MilestoneState::Closed,
                                        _ => MilestoneState::Open,
                                    },
                                    open_issues: m.open_issues as u32,
                                    closed_issues: m.closed_issues as u32,
                                    due_on: m.due_on,
                                    created_at: m.created_at,
                                    closed_at: m.closed_at,
                                }),
                            None => None,
                        };

                    issues.push(Issue {
                        id: row.id,
                        number: row.number as u64,
                        title: row.title,
                        body: row.body,
                        state: match row.state.as_str() {
                            "closed" => IssueState::Closed,
                            _ => IssueState::Open,
                        },
                        labels,
                        assignees: vec![],
                        milestone: ms,
                        author: greport_core::models::User {
                            id: row.author_id,
                            login: row.author_login,
                            avatar_url: String::new(),
                            html_url: String::new(),
                        },
                        comments_count: row.comments_count as u32,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        closed_at: row.closed_at,
                        closed_by: None,
                    });
                }
            }

            if types.iter().any(|t| t == "milestones")
                && convert::has_synced_data(pool, repo_db_id, "milestones").await
            {
                let rows = greport_db::queries::list_milestones_in_date_range(
                    pool, repo_db_id, start_dt, end_dt,
                )
                .await?;
                milestones = rows
                    .into_iter()
                    .map(|m| Milestone {
                        id: m.id,
                        number: m.number as u64,
                        title: m.title,
                        description: m.description,
                        state: match m.state.as_str() {
                            "closed" => MilestoneState::Closed,
                            _ => MilestoneState::Open,
                        },
                        open_issues: m.open_issues as u32,
                        closed_issues: m.closed_issues as u32,
                        due_on: m.due_on,
                        created_at: m.created_at,
                        closed_at: m.closed_at,
                    })
                    .collect();
            }

            if types.iter().any(|t| t == "releases")
                && convert::has_synced_data(pool, repo_db_id, "releases").await
            {
                let rows = greport_db::queries::list_releases_in_date_range(
                    pool, repo_db_id, start_dt, end_dt,
                )
                .await?;
                releases = rows
                    .into_iter()
                    .map(|r| Release {
                        id: r.id,
                        tag_name: r.tag_name,
                        name: r.name,
                        body: r.body,
                        draft: r.draft,
                        prerelease: r.prerelease,
                        author: greport_core::models::User {
                            id: r.author_id,
                            login: r.author_login,
                            avatar_url: String::new(),
                            html_url: String::new(),
                        },
                        created_at: r.created_at,
                        published_at: r.published_at,
                    })
                    .collect();
            }

            if types.iter().any(|t| t == "pulls")
                && convert::has_synced_data(pool, repo_db_id, "pulls").await
            {
                let rows = greport_db::queries::list_pulls_merged_in_date_range(
                    pool, repo_db_id, start_dt, end_dt,
                )
                .await?;
                pulls = rows
                    .into_iter()
                    .map(|p| PullRequest {
                        id: p.id,
                        number: p.number as u64,
                        title: p.title,
                        body: p.body,
                        state: match p.state.as_str() {
                            "closed" => greport_core::models::PullState::Closed,
                            _ => greport_core::models::PullState::Open,
                        },
                        draft: p.draft,
                        author: greport_core::models::User {
                            id: p.author_id,
                            login: p.author_login,
                            avatar_url: String::new(),
                            html_url: String::new(),
                        },
                        labels: vec![],
                        milestone: None,
                        head_ref: p.head_ref,
                        base_ref: p.base_ref,
                        merged: p.merged,
                        merged_at: p.merged_at,
                        additions: p.additions as u32,
                        deletions: p.deletions as u32,
                        changed_files: p.changed_files as u32,
                        created_at: p.created_at,
                        updated_at: p.updated_at,
                        closed_at: p.closed_at,
                    })
                    .collect();
            }

            let mut events = build_calendar_events(
                &issues,
                &milestones,
                &releases,
                &pulls,
                &repo_name,
                &types,
                start,
                end,
            );
            events.sort_by(|a, b| a.date.cmp(&b.date));
            let summary = compute_summary(&events);

            return Ok(Json(ApiResponse::ok(CalendarData {
                start_date: start,
                end_date: end,
                events,
                summary,
            })));
        }
    }

    // Fallback: GitHub API (fetch all data and filter in-memory)
    // Individual API calls are wrapped to handle partial failures gracefully
    // (e.g., token lacking release permissions should not block issues/milestones)
    use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};

    let repo_id = RepoId::new(owner, repo);
    let mut issues = Vec::new();
    let mut milestones = Vec::new();
    let mut releases = Vec::new();
    let mut pulls = Vec::new();

    if types.iter().any(|t| t == "issues") {
        match state.github.list_issues(&repo_id, IssueParams::all()).await {
            Ok(data) => issues = data,
            Err(e) => tracing::warn!(
                "Calendar fallback: failed to fetch issues for {}: {}",
                repo_name,
                e
            ),
        }
    }
    if types.iter().any(|t| t == "milestones") {
        match state.github.list_milestones(&repo_id).await {
            Ok(data) => milestones = data,
            Err(e) => tracing::warn!(
                "Calendar fallback: failed to fetch milestones for {}: {}",
                repo_name,
                e
            ),
        }
    }
    if types.iter().any(|t| t == "releases") {
        match state.github.list_releases(&repo_id).await {
            Ok(data) => releases = data,
            Err(e) => tracing::warn!(
                "Calendar fallback: failed to fetch releases for {}: {}",
                repo_name,
                e
            ),
        }
    }
    if types.iter().any(|t| t == "pulls") {
        match state.github.list_pulls(&repo_id, PullParams::all()).await {
            Ok(all_pulls) => pulls = all_pulls.into_iter().filter(|p| p.merged).collect(),
            Err(e) => tracing::warn!(
                "Calendar fallback: failed to fetch pulls for {}: {}",
                repo_name,
                e
            ),
        }
    }

    let mut events = build_calendar_events(
        &issues,
        &milestones,
        &releases,
        &pulls,
        &repo_name,
        &types,
        start,
        end,
    );
    events.sort_by(|a, b| a.date.cmp(&b.date));
    let summary = compute_summary(&events);

    Ok(Json(ApiResponse::ok(CalendarData {
        start_date: start,
        end_date: end,
        events,
        summary,
    })))
}

/// GET /api/v1/aggregate/calendar
pub async fn get_aggregate_calendar(
    State(state): State<AppState>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<ApiResponse<CalendarData>>, ApiError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for aggregate calendar".into()))?;

    let (default_start, default_end) = default_date_range();
    let start = query
        .start_date
        .as_deref()
        .and_then(parse_date)
        .unwrap_or(default_start);
    let end = query
        .end_date
        .as_deref()
        .and_then(parse_date)
        .unwrap_or(default_end);
    let types = parse_types(query.types.as_deref());

    let start_dt = Utc.from_utc_datetime(&start.and_hms_opt(0, 0, 0).unwrap());
    let end_dt = Utc.from_utc_datetime(&end.and_hms_opt(23, 59, 59).unwrap());

    let tracked = greport_db::queries::list_tracked_repos(pool).await?;
    let mut all_events = Vec::new();

    for tracked_repo in &tracked {
        let parts: Vec<&str> = tracked_repo.full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            continue;
        }
        let repo_owner = parts[0];
        let repo_name_part = parts[1];
        let repo_full_name = &tracked_repo.full_name;

        let repo_db_id = match convert::get_repo_db_id(pool, repo_owner, repo_name_part).await {
            Some(id) => id,
            None => continue,
        };

        let mut issues = Vec::new();
        let mut milestones_list = Vec::new();
        let mut releases_list = Vec::new();
        let mut pulls_list = Vec::new();

        if types.iter().any(|t| t == "issues")
            && convert::has_synced_data(pool, repo_db_id, "issues").await
        {
            let rows =
                greport_db::queries::list_issues_in_date_range(pool, repo_db_id, start_dt, end_dt)
                    .await?;
            for row in rows {
                let label_rows = greport_db::queries::get_issue_labels(pool, row.id).await?;
                let labels: Vec<greport_core::models::Label> = label_rows
                    .into_iter()
                    .map(|l| greport_core::models::Label {
                        id: l.label_id,
                        name: l.label_name,
                        color: l.label_color.unwrap_or_default(),
                        description: None,
                    })
                    .collect();

                issues.push(Issue {
                    id: row.id,
                    number: row.number as u64,
                    title: row.title,
                    body: row.body,
                    state: match row.state.as_str() {
                        "closed" => IssueState::Closed,
                        _ => IssueState::Open,
                    },
                    labels,
                    assignees: vec![],
                    milestone: None,
                    author: greport_core::models::User {
                        id: row.author_id,
                        login: row.author_login,
                        avatar_url: String::new(),
                        html_url: String::new(),
                    },
                    comments_count: row.comments_count as u32,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    closed_at: row.closed_at,
                    closed_by: None,
                });
            }
        }

        if types.iter().any(|t| t == "milestones")
            && convert::has_synced_data(pool, repo_db_id, "milestones").await
        {
            let rows = greport_db::queries::list_milestones_in_date_range(
                pool, repo_db_id, start_dt, end_dt,
            )
            .await?;
            milestones_list = rows
                .into_iter()
                .map(|m| Milestone {
                    id: m.id,
                    number: m.number as u64,
                    title: m.title,
                    description: m.description,
                    state: match m.state.as_str() {
                        "closed" => MilestoneState::Closed,
                        _ => MilestoneState::Open,
                    },
                    open_issues: m.open_issues as u32,
                    closed_issues: m.closed_issues as u32,
                    due_on: m.due_on,
                    created_at: m.created_at,
                    closed_at: m.closed_at,
                })
                .collect();
        }

        if types.iter().any(|t| t == "releases")
            && convert::has_synced_data(pool, repo_db_id, "releases").await
        {
            let rows = greport_db::queries::list_releases_in_date_range(
                pool, repo_db_id, start_dt, end_dt,
            )
            .await?;
            releases_list = rows
                .into_iter()
                .map(|r| Release {
                    id: r.id,
                    tag_name: r.tag_name,
                    name: r.name,
                    body: r.body,
                    draft: r.draft,
                    prerelease: r.prerelease,
                    author: greport_core::models::User {
                        id: r.author_id,
                        login: r.author_login,
                        avatar_url: String::new(),
                        html_url: String::new(),
                    },
                    created_at: r.created_at,
                    published_at: r.published_at,
                })
                .collect();
        }

        if types.iter().any(|t| t == "pulls")
            && convert::has_synced_data(pool, repo_db_id, "pulls").await
        {
            let rows = greport_db::queries::list_pulls_merged_in_date_range(
                pool, repo_db_id, start_dt, end_dt,
            )
            .await?;
            pulls_list = rows
                .into_iter()
                .map(|p| PullRequest {
                    id: p.id,
                    number: p.number as u64,
                    title: p.title,
                    body: p.body,
                    state: match p.state.as_str() {
                        "closed" => greport_core::models::PullState::Closed,
                        _ => greport_core::models::PullState::Open,
                    },
                    draft: p.draft,
                    author: greport_core::models::User {
                        id: p.author_id,
                        login: p.author_login,
                        avatar_url: String::new(),
                        html_url: String::new(),
                    },
                    labels: vec![],
                    milestone: None,
                    head_ref: p.head_ref,
                    base_ref: p.base_ref,
                    merged: p.merged,
                    merged_at: p.merged_at,
                    additions: p.additions as u32,
                    deletions: p.deletions as u32,
                    changed_files: p.changed_files as u32,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                    closed_at: p.closed_at,
                })
                .collect();
        }

        let events = build_calendar_events(
            &issues,
            &milestones_list,
            &releases_list,
            &pulls_list,
            repo_full_name,
            &types,
            start,
            end,
        );
        all_events.extend(events);
    }

    all_events.sort_by(|a, b| a.date.cmp(&b.date));
    let summary = compute_summary(&all_events);

    Ok(Json(ApiResponse::ok(CalendarData {
        start_date: start,
        end_date: end,
        events: all_events,
        summary,
    })))
}
