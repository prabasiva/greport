import type {
  CalendarEvent,
  CalendarEventType,
  UpcomingRelease,
  RecentRelease,
  TimelineEntry,
  ReleasePlan,
  ReleasePlanStatus,
  Milestone,
  Release,
} from "@/types/api";

let idCounter = 0;
function nextId(): string {
  return `test-${++idCounter}`;
}

export function createCalendarEvent(
  overrides?: Partial<CalendarEvent>,
): CalendarEvent {
  return {
    id: nextId(),
    event_type: "issue_created" as CalendarEventType,
    title: "Test event",
    date: new Date().toISOString(),
    number: 1,
    state: "open",
    repository: "owner/repo",
    labels: [],
    milestone: undefined,
    url: "https://github.com/owner/repo/issues/1",
    ...overrides,
  };
}

export function createMilestone(overrides?: Partial<Milestone>): Milestone {
  return {
    id: 1,
    number: 1,
    title: "v1.0",
    description: null,
    state: "open",
    open_issues: 5,
    closed_issues: 15,
    due_on: new Date(Date.now() + 30 * 86400000).toISOString(),
    created_at: new Date(Date.now() - 60 * 86400000).toISOString(),
    closed_at: null,
    ...overrides,
  } as Milestone;
}

export function createRelease(overrides?: Partial<Release>): Release {
  return {
    id: 1,
    tag_name: "v1.0.0",
    name: "Release v1.0.0",
    body: "Release notes",
    draft: false,
    prerelease: false,
    author: { id: 1, login: "testuser", avatar_url: "", html_url: "" },
    created_at: new Date().toISOString(),
    published_at: new Date().toISOString(),
    ...overrides,
  } as Release;
}

export function createUpcomingRelease(
  overrides?: Partial<UpcomingRelease>,
): UpcomingRelease {
  return {
    milestone: createMilestone(),
    repository: "owner/repo",
    progress_percent: 75,
    days_remaining: 30,
    blocker_count: 0,
    status: "on_track" as ReleasePlanStatus,
    ...overrides,
  };
}

export function createRecentRelease(
  overrides?: Partial<RecentRelease>,
): RecentRelease {
  return {
    release: createRelease(),
    repository: "owner/repo",
    release_type: "stable",
    ...overrides,
  } as RecentRelease;
}

export function createTimelineEntry(
  overrides?: Partial<TimelineEntry>,
): TimelineEntry {
  return {
    date: new Date().toISOString(),
    entry_type: "release",
    title: "v1.0.0",
    repository: "owner/repo",
    is_future: false,
    progress_percent: undefined,
    ...overrides,
  } as TimelineEntry;
}

export function createReleasePlan(
  overrides?: Partial<ReleasePlan>,
): ReleasePlan {
  return {
    upcoming: [],
    recent_releases: [],
    timeline: [],
    ...overrides,
  };
}
