"use client";

import useSWR from "swr";
import {
  issuesUrl,
  issueMetricsUrl,
  velocityUrl,
  burndownUrl,
  staleIssuesUrl,
  pullsUrl,
  pullMetricsUrl,
  releasesUrl,
  releaseNotesUrl,
  slaUrl,
  contributorsUrl,
  orgsUrl,
  aggregateIssuesUrl,
  aggregatePullsUrl,
  aggregateIssueMetricsUrl,
  aggregatePullMetricsUrl,
  aggregateContributorsUrl,
  aggregateVelocityUrl,
  calendarUrl,
  aggregateCalendarUrl,
  releasePlanUrl,
  aggregateReleasePlanUrl,
} from "@/lib/api";
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
  SlaReportResponse,
  ContributorStats,
  OrgsListResponse,
  AggregateIssueItem,
  AggregatePullItem,
  AggregateIssueMetrics,
  AggregatePullMetrics,
  AggregateContributorStats,
  AggregateVelocityMetrics,
  CalendarData,
  ReleasePlan,
} from "@/types/api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:9423";

async function directFetcher<T>(url: string): Promise<T> {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error(body?.error?.message || res.statusText);
  }
  return res.json();
}

function useApi<T>(path: string | null) {
  return useSWR<T>(path ? `${API_BASE}${path}` : null, directFetcher as (url: string) => Promise<T>, {
    revalidateOnFocus: false,
    dedupingInterval: 30000,
  });
}

export function useIssues(
  owner: string,
  repo: string,
  params?: { state?: string; labels?: string; assignee?: string; milestone?: string; page?: number; per_page?: number; days?: number },
) {
  return useApi<PaginatedResponse<Issue>>(
    owner && repo ? issuesUrl(owner, repo, params) : null,
  );
}

export function useIssueMetrics(
  owner: string,
  repo: string,
  params?: { state?: string; days?: number },
) {
  return useApi<ApiResponse<IssueMetrics>>(
    owner && repo ? issueMetricsUrl(owner, repo, params) : null,
  );
}

export function useVelocity(
  owner: string,
  repo: string,
  params?: { period?: string; last?: number },
) {
  return useApi<ApiResponse<VelocityMetrics>>(
    owner && repo ? velocityUrl(owner, repo, params) : null,
  );
}

export function useBurndown(owner: string, repo: string, milestone: string | null) {
  return useApi<ApiResponse<BurndownReport>>(
    owner && repo && milestone ? burndownUrl(owner, repo, milestone) : null,
  );
}

export function useStaleIssues(owner: string, repo: string, days?: number) {
  return useApi<ApiResponse<Issue[]>>(
    owner && repo ? staleIssuesUrl(owner, repo, days) : null,
  );
}

export function usePulls(
  owner: string,
  repo: string,
  params?: { state?: string; page?: number; per_page?: number; days?: number },
) {
  return useApi<PaginatedResponse<PullRequest>>(
    owner && repo ? pullsUrl(owner, repo, params) : null,
  );
}

export function usePullMetrics(
  owner: string,
  repo: string,
  params?: { state?: string; days?: number },
) {
  return useApi<ApiResponse<PullMetrics>>(
    owner && repo ? pullMetricsUrl(owner, repo, params) : null,
  );
}

export function useReleases(
  owner: string,
  repo: string,
  params?: { page?: number; per_page?: number },
) {
  return useApi<PaginatedResponse<Release>>(
    owner && repo ? releasesUrl(owner, repo, params) : null,
  );
}

export function useReleaseNotes(
  owner: string,
  repo: string,
  milestone: string | null,
  version?: string,
) {
  return useApi<ApiResponse<ReleaseNotes>>(
    owner && repo && milestone
      ? releaseNotesUrl(owner, repo, milestone, version)
      : null,
  );
}

export function useSla(
  owner: string,
  repo: string,
  params?: { response_hours?: number; resolution_hours?: number },
) {
  return useApi<ApiResponse<SlaReportResponse>>(
    owner && repo ? slaUrl(owner, repo, params) : null,
  );
}

export function useContributors(
  owner: string,
  repo: string,
  params?: { sort_by?: string; limit?: number },
) {
  return useApi<ApiResponse<ContributorStats[]>>(
    owner && repo ? contributorsUrl(owner, repo, params) : null,
  );
}


// Organizations

export function useOrgs() {
  return useApi<ApiResponse<OrgsListResponse>>(orgsUrl());
}

// Aggregate hooks

export function useAggregateIssues(
  params?: { state?: string; days?: number; page?: number; per_page?: number },
) {
  return useApi<PaginatedResponse<AggregateIssueItem>>(
    aggregateIssuesUrl(params),
  );
}

export function useAggregatePulls(
  params?: { state?: string; days?: number; page?: number; per_page?: number },
) {
  return useApi<PaginatedResponse<AggregatePullItem>>(
    aggregatePullsUrl(params),
  );
}

export function useAggregateIssueMetrics(params?: { state?: string; days?: number }) {
  return useApi<ApiResponse<AggregateIssueMetrics>>(
    aggregateIssueMetricsUrl(params),
  );
}

export function useAggregatePullMetrics(params?: { state?: string; days?: number }) {
  return useApi<ApiResponse<AggregatePullMetrics>>(
    aggregatePullMetricsUrl(params),
  );
}

export function useAggregateContributors() {
  return useApi<ApiResponse<AggregateContributorStats[]>>(
    aggregateContributorsUrl(),
  );
}

export function useAggregateVelocity(
  params?: { period?: string; last?: number },
) {
  return useApi<ApiResponse<AggregateVelocityMetrics>>(
    aggregateVelocityUrl(params),
  );
}

// Calendar hooks

export function useCalendar(
  owner: string,
  repo: string,
  params?: { start_date?: string; end_date?: string; types?: string },
) {
  return useApi<ApiResponse<CalendarData>>(
    owner && repo ? calendarUrl(owner, repo, params) : null,
  );
}

export function useAggregateCalendar(
  params?: { start_date?: string; end_date?: string; types?: string },
) {
  return useApi<ApiResponse<CalendarData>>(
    aggregateCalendarUrl(params),
  );
}

// Release Plan hooks

export function useReleasePlan(
  owner: string,
  repo: string,
  params?: { months_back?: number; months_forward?: number },
) {
  return useApi<ApiResponse<ReleasePlan>>(
    owner && repo ? releasePlanUrl(owner, repo, params) : null,
  );
}

export function useAggregateReleasePlan(
  params?: { months_back?: number; months_forward?: number },
) {
  return useApi<ApiResponse<ReleasePlan>>(
    aggregateReleasePlanUrl(params),
  );
}
