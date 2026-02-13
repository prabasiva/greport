// Core models matching the greport-api response types

export interface User {
  id: number;
  login: string;
  avatar_url: string;
  html_url: string;
}

export interface Label {
  id: number;
  name: string;
  color: string;
  description?: string;
}

export type IssueState = "open" | "closed";
export type MilestoneState = "open" | "closed";
export type PullState = "open" | "closed";
export type PrSize = "XS" | "S" | "M" | "L" | "XL";
export type Period = "day" | "week" | "month";
export type Trend = "increasing" | "decreasing" | "stable";
export type ViolationType = "Response" | "Resolution";

export interface Milestone {
  id: number;
  number: number;
  title: string;
  description?: string;
  state: MilestoneState;
  open_issues: number;
  closed_issues: number;
  due_on?: string;
  created_at: string;
  closed_at?: string;
}

export interface Issue {
  id: number;
  number: number;
  title: string;
  body?: string;
  state: IssueState;
  labels: Label[];
  assignees: User[];
  milestone?: Milestone;
  author: User;
  comments_count: number;
  created_at: string;
  updated_at: string;
  closed_at?: string;
  closed_by?: User;
}

export interface PullRequest {
  id: number;
  number: number;
  title: string;
  body?: string;
  state: PullState;
  draft: boolean;
  author: User;
  labels: Label[];
  milestone?: Milestone;
  head_ref: string;
  base_ref: string;
  merged: boolean;
  merged_at?: string;
  additions: number;
  deletions: number;
  changed_files: number;
  created_at: string;
  updated_at: string;
  closed_at?: string;
}

export interface Release {
  id: number;
  tag_name: string;
  name?: string;
  body?: string;
  draft: boolean;
  prerelease: boolean;
  author: User;
  created_at: string;
  published_at?: string;
}

// Metrics types

export interface AgeBucket {
  label: string;
  min_days: number;
  max_days?: number;
  count: number;
}

export interface AgeDistribution {
  buckets: AgeBucket[];
}

export interface IssueMetrics {
  total: number;
  open: number;
  closed: number;
  avg_time_to_close_hours?: number;
  median_time_to_close_hours?: number;
  by_label: Record<string, number>;
  by_assignee: Record<string, number>;
  by_milestone: Record<string, number>;
  age_distribution: AgeDistribution;
  stale_count: number;
}

export interface PullMetrics {
  total: number;
  open: number;
  merged: number;
  closed_unmerged: number;
  avg_time_to_merge_hours?: number;
  median_time_to_merge_hours?: number;
  by_size: Record<string, number>;
  by_author: Record<string, number>;
  by_base_branch: Record<string, number>;
  draft_count: number;
}

export interface VelocityDataPoint {
  period_start: string;
  period_end: string;
  opened: number;
  closed: number;
  net_change: number;
  cumulative_open: number;
}

export interface VelocityMetrics {
  period: Period;
  data_points: VelocityDataPoint[];
  avg_opened: number;
  avg_closed: number;
  trend: Trend;
}

// Burndown / Burnup

export interface BurndownDataPoint {
  date: string;
  remaining: number;
  completed: number;
}

export interface BurndownReport {
  milestone: string;
  start_date: string;
  end_date?: string;
  total_issues: number;
  data_points: BurndownDataPoint[];
  ideal_burndown: BurndownDataPoint[];
  projected_completion?: string;
}

// Release Notes

export interface ReleaseItem {
  number: number;
  title: string;
  author: string;
  labels: string[];
}

export interface ReleaseSection {
  title: string;
  items: ReleaseItem[];
}

export interface ReleaseStats {
  issues_closed: number;
  prs_merged: number;
  contributors_count: number;
}

export interface ReleaseNotes {
  version: string;
  date: string;
  summary: string;
  sections: ReleaseSection[];
  contributors: string[];
  stats: ReleaseStats;
}

// SLA

export type SlaStatus =
  | { type: "Ok" }
  | { type: "AtRisk"; percent_elapsed: number }
  | { type: "ResponseBreached"; hours_overdue: number }
  | { type: "ResolutionBreached"; hours_overdue: number };

export interface SlaIssue {
  number: number;
  title: string;
  url: string;
  author: string;
  created_at: string;
  age_hours: number;
  sla_status: SlaStatus;
  labels: string[];
}

export interface SlaSummary {
  total_open: number;
  within_sla: number;
  response_breached: number;
  resolution_breached: number;
  at_risk: number;
  compliance_rate: number;
}

export interface SlaConfig {
  response_time_hours: number;
  resolution_time_hours: number;
}

export interface SlaReportResponse {
  repository: string;
  config: SlaConfig;
  summary: SlaSummary;
  breaching_issues: SlaIssue[];
  at_risk_issues: SlaIssue[];
  generated_at: string;
}

// Contributor

export interface ContributorStats {
  login: string;
  issues_created: number;
  prs_created: number;
  prs_merged: number;
}

// Sync

export interface SyncResult {
  repository: string;
  issues_synced: number;
  pulls_synced: number;
  releases_synced: number;
  milestones_synced: number;
  synced_at: string;
  warnings?: string[];
}

// Multi-repo types

export interface SyncStatusSummary {
  issues_synced: boolean;
  pulls_synced: boolean;
  releases_synced: boolean;
  milestones_synced: boolean;
  last_synced_at?: string;
}

export interface RepoSummary {
  owner: string;
  name: string;
  full_name: string;
  description?: string;
  sync_status?: SyncStatusSummary;
}

export interface RepoSyncResult {
  repository: string;
  success: boolean;
  issues_synced?: number;
  pulls_synced?: number;
  releases_synced?: number;
  milestones_synced?: number;
  error?: string;
  warnings?: string[];
}

export interface BatchSyncResult {
  results: RepoSyncResult[];
  total_repos: number;
  successful: number;
  failed: number;
  synced_at: string;
}

// Organizations

export interface OrgSummary {
  name: string;
  web_url: string;
  repo_count: number;
}

export interface OrgsListResponse {
  orgs: OrgSummary[];
  default_web_url: string;
}

// Aggregate metrics

export interface RepoIssueMetrics {
  repository: string;
  total: number;
  open: number;
  closed: number;
  avg_time_to_close_hours?: number;
  stale_count: number;
}

export interface IssueMetricsTotals {
  total: number;
  open: number;
  closed: number;
  avg_time_to_close_hours?: number;
  stale_count: number;
}

export interface AggregateIssueMetrics {
  by_repository: RepoIssueMetrics[];
  totals: IssueMetricsTotals;
  by_label: Record<string, number>;
  by_assignee: Record<string, number>;
  age_distribution: AgeBucket[];
}

export interface RepoPullMetrics {
  repository: string;
  total: number;
  open: number;
  merged: number;
  avg_time_to_merge_hours?: number;
}

export interface PullMetricsTotals {
  total: number;
  open: number;
  merged: number;
  avg_time_to_merge_hours?: number;
}

export interface AggregatePullMetrics {
  by_repository: RepoPullMetrics[];
  totals: PullMetricsTotals;
  by_size: Record<string, number>;
  by_author: Record<string, number>;
}

export interface AggregateContributorStats {
  login: string;
  repositories: string[];
  total_issues_created: number;
  total_prs_created: number;
  total_prs_merged: number;
}

export interface RepoVelocityEntry {
  repository: string;
  avg_opened: number;
  avg_closed: number;
}

export interface AggregateVelocityMetrics {
  period: string;
  by_repository: RepoVelocityEntry[];
  combined_avg_opened: number;
  combined_avg_closed: number;
  trend: string;
}

// Aggregate list item types (issue/pull with repository field via serde flatten)

export type AggregateIssueItem = Issue & { repository: string };
export type AggregatePullItem = PullRequest & { repository: string };

// Calendar types

export type CalendarEventType =
  | "issue_created"
  | "issue_closed"
  | "milestone_due"
  | "milestone_closed"
  | "release_published"
  | "pr_merged";

export interface CalendarEvent {
  id: string;
  event_type: CalendarEventType;
  title: string;
  date: string;
  number?: number;
  state?: string;
  repository: string;
  labels: string[];
  milestone?: string;
  url: string;
}

export interface CalendarSummary {
  total_events: number;
  by_type: Record<string, number>;
}

export interface CalendarData {
  start_date: string;
  end_date: string;
  events: CalendarEvent[];
  summary: CalendarSummary;
}

// Release Plan types

export type ReleasePlanStatus = "on_track" | "at_risk" | "overdue";

export interface UpcomingRelease {
  milestone: Milestone;
  repository: string;
  progress_percent: number;
  days_remaining: number;
  blocker_count: number;
  status: ReleasePlanStatus;
}

export interface RecentRelease {
  release: Release;
  repository: string;
  release_type: "stable" | "prerelease" | "draft";
}

export interface TimelineEntry {
  date: string;
  entry_type: "release" | "milestone";
  title: string;
  repository: string;
  is_future: boolean;
  progress_percent?: number;
}

export interface ReleasePlan {
  upcoming: UpcomingRelease[];
  recent_releases: RecentRelease[];
  timeline: TimelineEntry[];
}

// GitHub Projects V2 types

export interface ProjectSummary {
  number: number;
  owner: string;
  title: string;
  description?: string;
  url: string;
  closed: boolean;
  total_items: number;
  synced_at: string;
}

export interface ProjectFieldSummary {
  name: string;
  field_type: string;
  config_json?: unknown;
}

export interface ProjectDetail extends ProjectSummary {
  fields: ProjectFieldSummary[];
}

export interface ProjectItemResponse {
  node_id: string;
  content_type: string;
  content_number?: number;
  content_title: string;
  content_state?: string;
  content_url?: string;
  content_repository?: string;
  field_values?: unknown;
}

export interface StatusCount {
  status: string;
  count: number;
}

export interface ContentTypeCount {
  content_type: string;
  count: number;
}

export interface ProjectMetrics {
  project_number: number;
  project_title: string;
  total_items: number;
  by_status: StatusCount[];
  by_content_type: ContentTypeCount[];
}

// API Response wrappers

export interface ApiResponse<T> {
  data: T;
}

export interface PaginationMeta {
  page: number;
  per_page: number;
  total: number;
  total_pages: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: PaginationMeta;
}

export interface ErrorBody {
  code: string;
  message: string;
}

export interface ErrorResponse {
  error: ErrorBody;
}
