import type {
  ApiResponse,
  PaginatedResponse,
  Issue,
  IssueMetrics,
  VelocityMetrics,
  BurndownReport,
  PullRequest,
  PullMetrics,
  Release,
  ReleaseNotes,
  Milestone,
  SlaReportResponse,
  ContributorStats,
  SyncResult,
  ErrorResponse,
  RepoSummary,
  BatchSyncResult,
  ProjectSummary,
  ProjectDetail,
  ProjectItemResponse,
  ProjectMetrics,
  ProjectFieldSummary,
  StatusCount,
  ContentTypeCount,
} from "@/types/api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:9423";

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const url = `${API_BASE}${path}`;
  const res = await fetch(url, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!res.ok) {
    let errorBody: ErrorResponse | null = null;
    try {
      errorBody = await res.json();
    } catch {
      // ignore parse error
    }
    throw new ApiError(
      res.status,
      errorBody?.error?.code || "unknown",
      errorBody?.error?.message || res.statusText,
    );
  }

  return res.json();
}

function buildQuery(params: Record<string, string | number | undefined>): string {
  const entries = Object.entries(params).filter(
    ([, v]) => v !== undefined && v !== "",
  );
  if (entries.length === 0) return "";
  return "?" + new URLSearchParams(
    entries.map(([k, v]) => [k, String(v)]),
  ).toString();
}

// SWR fetcher
export const fetcher = <T>(url: string): Promise<T> => apiFetch<T>(url);

// Issues
export function issuesUrl(
  owner: string,
  repo: string,
  params?: {
    state?: string;
    labels?: string;
    assignee?: string;
    milestone?: string;
    page?: number;
    per_page?: number;
    days?: number;
  },
): string {
  return `/api/v1/repos/${owner}/${repo}/issues${buildQuery(params || {})}`;
}

export function issueMetricsUrl(
  owner: string,
  repo: string,
  params?: { state?: string; days?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/issues/metrics${buildQuery(params || {})}`;
}

export function velocityUrl(
  owner: string,
  repo: string,
  params?: { period?: string; last?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/issues/velocity${buildQuery(params || {})}`;
}

export function burndownUrl(
  owner: string,
  repo: string,
  milestone: string,
): string {
  return `/api/v1/repos/${owner}/${repo}/issues/burndown${buildQuery({ milestone })}`;
}

export function staleIssuesUrl(
  owner: string,
  repo: string,
  days?: number,
): string {
  return `/api/v1/repos/${owner}/${repo}/issues/stale${buildQuery({ days })}`;
}

// Pulls
export function pullsUrl(
  owner: string,
  repo: string,
  params?: { state?: string; page?: number; per_page?: number; days?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/pulls${buildQuery(params || {})}`;
}

export function pullMetricsUrl(
  owner: string,
  repo: string,
  params?: { state?: string; days?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/pulls/metrics${buildQuery(params || {})}`;
}

// Releases
export function releasesUrl(
  owner: string,
  repo: string,
  params?: { page?: number; per_page?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/releases${buildQuery(params || {})}`;
}

export function releaseNotesUrl(
  owner: string,
  repo: string,
  milestone: string,
  version?: string,
): string {
  return `/api/v1/repos/${owner}/${repo}/releases/notes${buildQuery({ milestone, version })}`;
}

export function milestoneProgressUrl(
  owner: string,
  repo: string,
  milestone: string,
): string {
  return `/api/v1/repos/${owner}/${repo}/milestones/${encodeURIComponent(milestone)}/progress`;
}

// SLA
export function slaUrl(
  owner: string,
  repo: string,
  params?: { response_hours?: number; resolution_hours?: number; labels?: string },
): string {
  return `/api/v1/repos/${owner}/${repo}/sla${buildQuery(params || {})}`;
}

// Contributors
export function contributorsUrl(
  owner: string,
  repo: string,
  params?: { sort_by?: string; limit?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/contributors${buildQuery(params || {})}`;
}

// Sync
export function syncUrl(owner: string, repo: string): string {
  return `/api/v1/repos/${owner}/${repo}/sync`;
}

export async function syncRepo(
  owner: string,
  repo: string,
): Promise<ApiResponse<SyncResult>> {
  return apiFetch<ApiResponse<SyncResult>>(syncUrl(owner, repo), {
    method: "POST",
  });
}

// Repository management
export function reposUrl(): string {
  return "/api/v1/repos";
}

export async function fetchRepos(): Promise<ApiResponse<RepoSummary[]>> {
  return apiFetch<ApiResponse<RepoSummary[]>>(reposUrl());
}

export async function addTrackedRepo(
  fullName: string,
): Promise<ApiResponse<RepoSummary>> {
  return apiFetch<ApiResponse<RepoSummary>>(reposUrl(), {
    method: "POST",
    body: JSON.stringify({ full_name: fullName }),
  });
}

export async function removeTrackedRepo(
  owner: string,
  repo: string,
): Promise<void> {
  await apiFetch<void>(`/api/v1/repos/${owner}/${repo}`, {
    method: "DELETE",
  });
}

// Batch sync
export function batchSyncUrl(): string {
  return "/api/v1/sync";
}

export async function batchSync(): Promise<ApiResponse<BatchSyncResult>> {
  return apiFetch<ApiResponse<BatchSyncResult>>(batchSyncUrl(), {
    method: "POST",
  });
}

// Organizations
export function orgsUrl(): string {
  return "/api/v1/orgs";
}

// Aggregate lists
export function aggregateIssuesUrl(params?: { state?: string; days?: number; page?: number; per_page?: number }): string {
  return `/api/v1/aggregate/issues${buildQuery(params || {})}`;
}

export function aggregatePullsUrl(params?: { state?: string; days?: number; page?: number; per_page?: number }): string {
  return `/api/v1/aggregate/pulls${buildQuery(params || {})}`;
}

// Aggregate metrics
export function aggregateIssueMetricsUrl(params?: { state?: string; days?: number }): string {
  return `/api/v1/aggregate/issues/metrics${buildQuery(params || {})}`;
}

export function aggregatePullMetricsUrl(params?: { state?: string; days?: number }): string {
  return `/api/v1/aggregate/pulls/metrics${buildQuery(params || {})}`;
}

export function aggregateContributorsUrl(): string {
  return "/api/v1/aggregate/contributors";
}

export function aggregateVelocityUrl(
  params?: { period?: string; last?: number },
): string {
  return `/api/v1/aggregate/velocity${buildQuery(params || {})}`;
}

// Calendar
export function calendarUrl(
  owner: string,
  repo: string,
  params?: { start_date?: string; end_date?: string; types?: string },
): string {
  return `/api/v1/repos/${owner}/${repo}/calendar${buildQuery(params || {})}`;
}

export function aggregateCalendarUrl(
  params?: { start_date?: string; end_date?: string; types?: string },
): string {
  return `/api/v1/aggregate/calendar${buildQuery(params || {})}`;
}

// Release Plan
export function releasePlanUrl(
  owner: string,
  repo: string,
  params?: { months_back?: number; months_forward?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/release-plan${buildQuery(params || {})}`;
}

export function aggregateReleasePlanUrl(
  params?: { months_back?: number; months_forward?: number },
): string {
  return `/api/v1/aggregate/release-plan${buildQuery(params || {})}`;
}

// Projects
export function projectsUrl(
  org: string,
  params?: { include_closed?: boolean },
): string {
  const query: Record<string, string | number | undefined> = {};
  if (params?.include_closed !== undefined) {
    query.include_closed = params.include_closed ? "true" : "false";
  }
  return `/api/v1/orgs/${org}/projects${buildQuery(query)}`;
}

export function projectDetailUrl(org: string, number: number): string {
  return `/api/v1/orgs/${org}/projects/${number}`;
}

export function projectItemsUrl(
  org: string,
  number: number,
  params?: { content_type?: string; state?: string; page?: number; per_page?: number },
): string {
  return `/api/v1/orgs/${org}/projects/${number}/items${buildQuery(params || {})}`;
}

export function projectMetricsUrl(org: string, number: number): string {
  return `/api/v1/orgs/${org}/projects/${number}/metrics`;
}

export function aggregateProjectsUrl(
  params?: { include_closed?: boolean },
): string {
  const query: Record<string, string | number | undefined> = {};
  if (params?.include_closed !== undefined) {
    query.include_closed = params.include_closed ? "true" : "false";
  }
  return `/api/v1/aggregate/projects${buildQuery(query)}`;
}

// Re-export types for convenience
export type {
  ApiResponse,
  PaginatedResponse,
  Issue,
  IssueMetrics,
  VelocityMetrics,
  BurndownReport,
  PullRequest,
  PullMetrics,
  Release,
  ReleaseNotes,
  Milestone,
  SlaReportResponse,
  ContributorStats,
  SyncResult,
  ProjectSummary,
  ProjectDetail,
  ProjectItemResponse,
  ProjectMetrics,
  ProjectFieldSummary,
  StatusCount,
  ContentTypeCount,
};
