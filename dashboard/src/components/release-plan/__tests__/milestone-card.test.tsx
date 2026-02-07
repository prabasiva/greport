import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { MilestoneCard } from "../milestone-card";
import { createUpcomingRelease, createMilestone } from "@/test/factories";

describe("MilestoneCard", () => {
  it("renders milestone title and repository", () => {
    const item = createUpcomingRelease({
      milestone: createMilestone({ title: "Sprint 5" }),
      repository: "org/project",
    });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("Sprint 5")).toBeInTheDocument();
    expect(screen.getByText("org/project")).toBeInTheDocument();
  });

  it("shows On Track status badge", () => {
    const item = createUpcomingRelease({ status: "on_track" });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("On Track")).toBeInTheDocument();
  });

  it("shows At Risk status badge", () => {
    const item = createUpcomingRelease({ status: "at_risk" });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("At Risk")).toBeInTheDocument();
  });

  it("shows Overdue status badge", () => {
    const item = createUpcomingRelease({ status: "overdue" });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("Overdue")).toBeInTheDocument();
  });

  it("shows blocker count when present", () => {
    const item = createUpcomingRelease({ blocker_count: 3 });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("3 blockers")).toBeInTheDocument();
  });

  it("hides blocker text when count is zero", () => {
    const item = createUpcomingRelease({ blocker_count: 0 });
    render(<MilestoneCard item={item} />);
    expect(screen.queryByText(/blocker/)).not.toBeInTheDocument();
  });

  it("shows days remaining for positive values", () => {
    const item = createUpcomingRelease({ days_remaining: 15 });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("15d remaining")).toBeInTheDocument();
  });

  it("shows Due today for zero days", () => {
    const item = createUpcomingRelease({ days_remaining: 0 });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("Due today")).toBeInTheDocument();
  });

  it("shows overdue label for negative days", () => {
    const item = createUpcomingRelease({ days_remaining: -5 });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("5d overdue")).toBeInTheDocument();
  });

  it("shows issue counts", () => {
    const item = createUpcomingRelease({
      milestone: createMilestone({ closed_issues: 10, open_issues: 5 }),
    });
    render(<MilestoneCard item={item} />);
    expect(screen.getByText("10 closed / 5 open")).toBeInTheDocument();
  });
});
