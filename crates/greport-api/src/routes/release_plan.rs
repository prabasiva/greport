//! Release plan route handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Months, Utc};
use serde::Deserialize;

use crate::convert;
use crate::error::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use greport_core::models::{
    Milestone, MilestoneState, RecentRelease, Release, ReleasePlan, ReleasePlanStatus,
    TimelineEntry, UpcomingRelease, User,
};

#[derive(Deserialize)]
pub struct ReleasePlanQuery {
    months_back: Option<u32>,
    months_forward: Option<u32>,
}

/// Classify a release as stable, prerelease, or draft
fn classify_release(release: &Release) -> String {
    if release.draft {
        "draft".to_string()
    } else if release.prerelease {
        "prerelease".to_string()
    } else {
        "stable".to_string()
    }
}

/// Compute milestone status based on progress and time remaining
fn compute_status(due_on: chrono::DateTime<Utc>, progress_percent: f64) -> ReleasePlanStatus {
    let now = Utc::now();
    if due_on < now {
        return ReleasePlanStatus::Overdue;
    }

    // Calculate percentage of time remaining
    let total_duration = due_on.signed_duration_since(now);
    let total_days = total_duration.num_days().max(1) as f64;
    // Estimate original duration from creation... use simpler heuristic:
    // If less than 25% time remaining but less than 75% done
    if total_days <= 7.0 && progress_percent < 75.0 {
        return ReleasePlanStatus::AtRisk;
    }

    ReleasePlanStatus::OnTrack
}

/// Build release plan data for a single repository from DB
async fn build_repo_release_plan(
    pool: &greport_db::DbPool,
    repo_db_id: i64,
    repo_full_name: &str,
    months_back: u32,
    months_forward: u32,
) -> Result<(Vec<UpcomingRelease>, Vec<RecentRelease>, Vec<TimelineEntry>), ApiError> {
    let now = Utc::now();
    let mut upcoming = Vec::new();
    let mut recent_releases = Vec::new();
    let mut timeline = Vec::new();

    // Forward date for milestones
    let forward_date = now
        .checked_add_months(Months::new(months_forward))
        .unwrap_or(now);

    // Backward date for releases
    let back_date = now
        .checked_sub_months(Months::new(months_back))
        .unwrap_or(now);

    // Fetch open milestones with due dates
    if convert::has_synced_data(pool, repo_db_id, "milestones").await {
        let ms_rows =
            greport_db::queries::list_open_milestones_with_due(pool, repo_db_id, forward_date)
                .await?;

        for m in ms_rows {
            let total = m.open_issues as f64 + m.closed_issues as f64;
            let progress = if total > 0.0 {
                (m.closed_issues as f64 / total) * 100.0
            } else {
                0.0
            };

            let days_remaining = m
                .due_on
                .map(|d| d.signed_duration_since(now).num_days())
                .unwrap_or(0);

            let blocker_count =
                greport_db::queries::count_blocker_issues(pool, repo_db_id, m.id).await? as usize;

            let status = m
                .due_on
                .map(|d| compute_status(d, progress))
                .unwrap_or(ReleasePlanStatus::OnTrack);

            let milestone = Milestone {
                id: m.id,
                number: m.number as u64,
                title: m.title,
                description: m.description,
                state: MilestoneState::Open,
                open_issues: m.open_issues as u32,
                closed_issues: m.closed_issues as u32,
                due_on: m.due_on,
                created_at: m.created_at,
                closed_at: m.closed_at,
            };

            // Add to timeline
            if let Some(due_on) = milestone.due_on {
                timeline.push(TimelineEntry {
                    date: due_on,
                    entry_type: "milestone".to_string(),
                    title: milestone.title.clone(),
                    repository: repo_full_name.to_string(),
                    is_future: due_on > now,
                    progress_percent: Some(progress),
                });
            }

            upcoming.push(UpcomingRelease {
                milestone,
                repository: repo_full_name.to_string(),
                progress_percent: progress,
                days_remaining,
                blocker_count,
                status,
            });
        }
    }

    // Fetch recent releases
    if convert::has_synced_data(pool, repo_db_id, "releases").await {
        let rel_rows =
            greport_db::queries::list_recent_releases(pool, repo_db_id, back_date).await?;

        for r in rel_rows {
            let release = Release {
                id: r.id,
                tag_name: r.tag_name,
                name: r.name,
                body: r.body,
                draft: r.draft,
                prerelease: r.prerelease,
                author: User {
                    id: r.author_id,
                    login: r.author_login,
                    avatar_url: String::new(),
                    html_url: String::new(),
                },
                created_at: r.created_at,
                published_at: r.published_at,
            };

            let release_type = classify_release(&release);

            // Add to timeline
            if let Some(published_at) = release.published_at {
                timeline.push(TimelineEntry {
                    date: published_at,
                    entry_type: "release".to_string(),
                    title: release
                        .name
                        .clone()
                        .unwrap_or_else(|| release.tag_name.clone()),
                    repository: repo_full_name.to_string(),
                    is_future: false,
                    progress_percent: None,
                });
            }

            recent_releases.push(RecentRelease {
                release,
                repository: repo_full_name.to_string(),
                release_type,
            });
        }
    }

    Ok((upcoming, recent_releases, timeline))
}

/// GET /api/v1/repos/{owner}/{repo}/release-plan
pub async fn get_release_plan(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleasePlanQuery>,
) -> Result<Json<ApiResponse<ReleasePlan>>, ApiError> {
    let months_back = query.months_back.unwrap_or(3);
    let months_forward = query.months_forward.unwrap_or(3);
    let repo_full_name = format!("{}/{}", owner, repo);

    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Database required for release plan".into()))?;

    let repo_db_id = convert::get_repo_db_id(pool, &owner, &repo)
        .await
        .ok_or_else(|| ApiError::NotFound("Repository not found in database".into()))?;

    let (mut upcoming, mut recent_releases, mut timeline) = build_repo_release_plan(
        pool,
        repo_db_id,
        &repo_full_name,
        months_back,
        months_forward,
    )
    .await?;

    upcoming.sort_by(|a, b| a.milestone.due_on.cmp(&b.milestone.due_on));
    recent_releases.sort_by(|a, b| b.release.published_at.cmp(&a.release.published_at));
    timeline.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(Json(ApiResponse::ok(ReleasePlan {
        upcoming,
        recent_releases,
        timeline,
    })))
}

/// GET /api/v1/aggregate/release-plan
pub async fn get_aggregate_release_plan(
    State(state): State<AppState>,
    Query(query): Query<ReleasePlanQuery>,
) -> Result<Json<ApiResponse<ReleasePlan>>, ApiError> {
    let months_back = query.months_back.unwrap_or(3);
    let months_forward = query.months_forward.unwrap_or(3);

    let pool = state.db.as_ref().ok_or_else(|| {
        ApiError::BadRequest("Database required for aggregate release plan".into())
    })?;

    let tracked = greport_db::queries::list_tracked_repos(pool).await?;
    let mut all_upcoming = Vec::new();
    let mut all_recent = Vec::new();
    let mut all_timeline = Vec::new();

    for tracked_repo in &tracked {
        let parts: Vec<&str> = tracked_repo.full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            continue;
        }
        let repo_owner = parts[0];
        let repo_name = parts[1];

        let repo_db_id = match convert::get_repo_db_id(pool, repo_owner, repo_name).await {
            Some(id) => id,
            None => continue,
        };

        match build_repo_release_plan(
            pool,
            repo_db_id,
            &tracked_repo.full_name,
            months_back,
            months_forward,
        )
        .await
        {
            Ok((upcoming, recent, timeline)) => {
                all_upcoming.extend(upcoming);
                all_recent.extend(recent);
                all_timeline.extend(timeline);
            }
            Err(e) => {
                tracing::warn!(
                    "Release plan: failed to build for {}: {}",
                    tracked_repo.full_name,
                    e
                );
            }
        }
    }

    all_upcoming.sort_by(|a, b| a.milestone.due_on.cmp(&b.milestone.due_on));
    all_recent.sort_by(|a, b| b.release.published_at.cmp(&a.release.published_at));
    all_timeline.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(Json(ApiResponse::ok(ReleasePlan {
        upcoming: all_upcoming,
        recent_releases: all_recent,
        timeline: all_timeline,
    })))
}
