import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { UpcomingReleases } from "../upcoming-releases";
import { createUpcomingRelease, createMilestone } from "@/test/factories";

describe("UpcomingReleases", () => {
  it("renders empty state when no items", () => {
    render(<UpcomingReleases items={[]} />);
    expect(
      screen.getByText("No upcoming milestones with due dates found."),
    ).toBeInTheDocument();
  });

  it("renders list of milestone cards", () => {
    const items = [
      createUpcomingRelease({
        milestone: createMilestone({ id: 1, title: "Sprint 1" }),
        repository: "org/repo-a",
      }),
      createUpcomingRelease({
        milestone: createMilestone({ id: 2, title: "Sprint 2" }),
        repository: "org/repo-b",
      }),
    ];
    render(<UpcomingReleases items={items} />);
    expect(screen.getByText("Sprint 1")).toBeInTheDocument();
    expect(screen.getByText("Sprint 2")).toBeInTheDocument();
  });
});
