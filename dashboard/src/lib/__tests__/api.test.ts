import { describe, it, expect } from "vitest";
import {
  calendarUrl,
  aggregateCalendarUrl,
  releasePlanUrl,
  aggregateReleasePlanUrl,
  issuesUrl,
  pullsUrl,
  releasesUrl,
  issueMetricsUrl,
  pullMetricsUrl,
  velocityUrl,
  burndownUrl,
  staleIssuesUrl,
  releaseNotesUrl,
  milestoneProgressUrl,
  slaUrl,
  contributorsUrl,
  syncUrl,
  reposUrl,
  batchSyncUrl,
  aggregateIssuesUrl,
  aggregatePullsUrl,
  aggregateIssueMetricsUrl,
  aggregatePullMetricsUrl,
  aggregateContributorsUrl,
  aggregateVelocityUrl,
} from "../api";

describe("URL builders", () => {
  describe("calendarUrl", () => {
    it("builds basic calendar path", () => {
      expect(calendarUrl("org", "repo")).toBe(
        "/api/v1/repos/org/repo/calendar",
      );
    });

    it("includes query params", () => {
      const url = calendarUrl("org", "repo", {
        start_date: "2025-01-01",
        end_date: "2025-03-31",
        types: "issues,releases",
      });
      expect(url).toContain("start_date=2025-01-01");
      expect(url).toContain("end_date=2025-03-31");
      expect(url).toContain("types=issues%2Creleases");
    });

    it("omits undefined params", () => {
      const url = calendarUrl("org", "repo", { start_date: "2025-01-01" });
      expect(url).toContain("start_date=2025-01-01");
      expect(url).not.toContain("end_date");
      expect(url).not.toContain("types");
    });
  });

  describe("aggregateCalendarUrl", () => {
    it("builds aggregate calendar path with no params", () => {
      expect(aggregateCalendarUrl()).toBe("/api/v1/aggregate/calendar");
    });

    it("includes query params", () => {
      const url = aggregateCalendarUrl({ start_date: "2025-01-01" });
      expect(url).toContain("start_date=2025-01-01");
    });
  });

  describe("releasePlanUrl", () => {
    it("builds basic release plan path", () => {
      expect(releasePlanUrl("org", "repo")).toBe(
        "/api/v1/repos/org/repo/release-plan",
      );
    });

    it("includes months_back and months_forward", () => {
      const url = releasePlanUrl("org", "repo", {
        months_back: 3,
        months_forward: 6,
      });
      expect(url).toContain("months_back=3");
      expect(url).toContain("months_forward=6");
    });
  });

  describe("aggregateReleasePlanUrl", () => {
    it("builds aggregate release plan path", () => {
      expect(aggregateReleasePlanUrl()).toBe(
        "/api/v1/aggregate/release-plan",
      );
    });

    it("includes params", () => {
      const url = aggregateReleasePlanUrl({ months_back: 2 });
      expect(url).toContain("months_back=2");
    });
  });

  describe("existing URL builders", () => {
    it("issuesUrl with params", () => {
      const url = issuesUrl("org", "repo", { state: "open", page: 2 });
      expect(url).toContain("/api/v1/repos/org/repo/issues");
      expect(url).toContain("state=open");
      expect(url).toContain("page=2");
    });

    it("pullsUrl", () => {
      expect(pullsUrl("org", "repo")).toBe("/api/v1/repos/org/repo/pulls");
    });

    it("releasesUrl", () => {
      expect(releasesUrl("org", "repo")).toBe(
        "/api/v1/repos/org/repo/releases",
      );
    });

    it("issueMetricsUrl", () => {
      expect(issueMetricsUrl("org", "repo")).toBe(
        "/api/v1/repos/org/repo/issues/metrics",
      );
    });

    it("pullMetricsUrl", () => {
      expect(pullMetricsUrl("org", "repo")).toBe(
        "/api/v1/repos/org/repo/pulls/metrics",
      );
    });

    it("velocityUrl", () => {
      const url = velocityUrl("org", "repo", { period: "weekly", last: 4 });
      expect(url).toContain("period=weekly");
      expect(url).toContain("last=4");
    });

    it("burndownUrl", () => {
      const url = burndownUrl("org", "repo", "v1.0");
      expect(url).toContain("milestone=v1.0");
    });

    it("staleIssuesUrl", () => {
      const url = staleIssuesUrl("org", "repo", 30);
      expect(url).toContain("days=30");
    });

    it("releaseNotesUrl", () => {
      const url = releaseNotesUrl("org", "repo", "v1.0", "1.0.0");
      expect(url).toContain("milestone=v1.0");
      expect(url).toContain("version=1.0.0");
    });

    it("milestoneProgressUrl encodes milestone name", () => {
      const url = milestoneProgressUrl("org", "repo", "Sprint 1");
      expect(url).toBe(
        "/api/v1/repos/org/repo/milestones/Sprint%201/progress",
      );
    });

    it("slaUrl", () => {
      const url = slaUrl("org", "repo", { response_hours: 24 });
      expect(url).toContain("response_hours=24");
    });

    it("contributorsUrl", () => {
      const url = contributorsUrl("org", "repo", { sort_by: "commits" });
      expect(url).toContain("sort_by=commits");
    });

    it("syncUrl", () => {
      expect(syncUrl("org", "repo")).toBe("/api/v1/repos/org/repo/sync");
    });

    it("reposUrl", () => {
      expect(reposUrl()).toBe("/api/v1/repos");
    });

    it("batchSyncUrl", () => {
      expect(batchSyncUrl()).toBe("/api/v1/sync");
    });

    it("aggregateIssuesUrl", () => {
      expect(aggregateIssuesUrl()).toBe("/api/v1/aggregate/issues");
    });

    it("aggregatePullsUrl", () => {
      expect(aggregatePullsUrl()).toBe("/api/v1/aggregate/pulls");
    });

    it("aggregateIssueMetricsUrl", () => {
      expect(aggregateIssueMetricsUrl()).toBe(
        "/api/v1/aggregate/issues/metrics",
      );
    });

    it("aggregatePullMetricsUrl", () => {
      expect(aggregatePullMetricsUrl()).toBe(
        "/api/v1/aggregate/pulls/metrics",
      );
    });

    it("aggregateContributorsUrl", () => {
      expect(aggregateContributorsUrl()).toBe(
        "/api/v1/aggregate/contributors",
      );
    });

    it("aggregateVelocityUrl", () => {
      const url = aggregateVelocityUrl({ period: "monthly" });
      expect(url).toContain("period=monthly");
    });
  });
});
