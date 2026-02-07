import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { RecentReleases } from "../recent-releases";
import { createRecentRelease, createRelease } from "@/test/factories";

describe("RecentReleases", () => {
  it("renders empty state when no items", () => {
    render(<RecentReleases items={[]} />);
    expect(
      screen.getByText("No releases published in the selected time range."),
    ).toBeInTheDocument();
  });

  it("renders list of release cards", () => {
    const items = [
      createRecentRelease({
        release: createRelease({ id: 1, name: "v1.0" }),
        repository: "org/repo",
      }),
      createRecentRelease({
        release: createRelease({ id: 2, name: "v1.1" }),
        repository: "org/repo",
      }),
    ];
    render(<RecentReleases items={items} />);
    expect(screen.getByText("v1.0")).toBeInTheDocument();
    expect(screen.getByText("v1.1")).toBeInTheDocument();
  });
});
