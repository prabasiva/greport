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
