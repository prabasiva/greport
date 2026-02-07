import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ReleaseCard } from "../release-card";
import { createRecentRelease, createRelease } from "@/test/factories";

describe("ReleaseCard", () => {
  it("renders release name and repository", () => {
    const item = createRecentRelease({
      release: createRelease({ name: "Big Release" }),
      repository: "org/project",
    });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Big Release")).toBeInTheDocument();
    expect(screen.getByText("org/project")).toBeInTheDocument();
  });

  it("shows tag name when name is missing", () => {
    const item = createRecentRelease({
      release: createRelease({ name: null, tag_name: "v2.0.0" }),
    });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("v2.0.0")).toBeInTheDocument();
  });

  it("shows Stable badge for stable releases", () => {
    const item = createRecentRelease({ release_type: "stable" });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Stable")).toBeInTheDocument();
  });

  it("shows Pre-release badge for prereleases", () => {
    const item = createRecentRelease({ release_type: "prerelease" });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Pre-release")).toBeInTheDocument();
  });

  it("shows Draft badge for drafts", () => {
    const item = createRecentRelease({ release_type: "draft" });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Draft")).toBeInTheDocument();
  });

  it("shows tag name in metadata", () => {
    const item = createRecentRelease({
      release: createRelease({ tag_name: "v3.1.0" }),
    });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Tag: v3.1.0")).toBeInTheDocument();
  });

  it("shows author login", () => {
    const item = createRecentRelease({
      release: createRelease({
        author: { id: 1, login: "jdoe", avatar_url: "", html_url: "" },
      }),
    });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("By: jdoe")).toBeInTheDocument();
  });

  it("shows body preview when present", () => {
    const item = createRecentRelease({
      release: createRelease({ body: "Fixed critical bugs and added features." }),
    });
    render(<ReleaseCard item={item} />);
    expect(screen.getByText("Fixed critical bugs and added features.")).toBeInTheDocument();
  });
});
