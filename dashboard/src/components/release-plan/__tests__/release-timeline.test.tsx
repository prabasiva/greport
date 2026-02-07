import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ReleaseTimeline } from "../release-timeline";
import { createTimelineEntry } from "@/test/factories";

describe("ReleaseTimeline", () => {
  it("renders empty state when no entries", () => {
    render(<ReleaseTimeline entries={[]} />);
    expect(
      screen.getByText("No timeline data available."),
    ).toBeInTheDocument();
  });

  it("renders markers for entries", () => {
    const entries = [
      createTimelineEntry({
        title: "v1.0",
        date: "2025-01-15T00:00:00Z",
        repository: "org/alpha",
        entry_type: "release",
      }),
      createTimelineEntry({
        title: "Sprint 3",
        date: "2025-02-10T00:00:00Z",
        repository: "org/beta",
        entry_type: "milestone",
      }),
    ];
    render(<ReleaseTimeline entries={entries} />);
    expect(screen.getByText("v1.0")).toBeInTheDocument();
    expect(screen.getByText("Sprint 3")).toBeInTheDocument();
  });

  it("renders legend with repo names", () => {
    const entries = [
      createTimelineEntry({
        title: "v1.0",
        date: "2025-01-15T00:00:00Z",
        repository: "org/alpha",
      }),
      createTimelineEntry({
        title: "v2.0",
        date: "2025-02-15T00:00:00Z",
        repository: "org/beta",
      }),
    ];
    render(<ReleaseTimeline entries={entries} />);
    // Repo names appear in tooltips and legend, so use getAllByText
    expect(screen.getAllByText("org/alpha").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("org/beta").length).toBeGreaterThanOrEqual(1);
  });

  it("renders legend items for Milestone and Release", () => {
    const entries = [
      createTimelineEntry({
        title: "v1.0",
        date: "2025-03-01T00:00:00Z",
      }),
    ];
    render(<ReleaseTimeline entries={entries} />);
    expect(screen.getByText("Milestone")).toBeInTheDocument();
    expect(screen.getByText("Release")).toBeInTheDocument();
  });

  it("shows month labels for the date range", () => {
    const entries = [
      createTimelineEntry({
        title: "v1.0",
        date: "2025-01-15T00:00:00Z",
      }),
      createTimelineEntry({
        title: "v2.0",
        date: "2025-03-15T00:00:00Z",
      }),
    ];
    render(<ReleaseTimeline entries={entries} />);
    expect(screen.getByText("Jan 2025")).toBeInTheDocument();
    expect(screen.getByText("Feb 2025")).toBeInTheDocument();
    expect(screen.getByText("Mar 2025")).toBeInTheDocument();
  });
});
