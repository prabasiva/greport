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
  ErrorResponse,
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
  },
): string {
  return `/api/v1/repos/${owner}/${repo}/issues${buildQuery(params || {})}`;
}

export function issueMetricsUrl(owner: string, repo: string): string {
  return `/api/v1/repos/${owner}/${repo}/issues/metrics`;
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
  params?: { state?: string; page?: number; per_page?: number },
): string {
  return `/api/v1/repos/${owner}/${repo}/pulls${buildQuery(params || {})}`;
}

export function pullMetricsUrl(owner: string, repo: string): string {
  return `/api/v1/repos/${owner}/${repo}/pulls/metrics`;
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
};
