import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ProgressBar } from "../progress-bar";

describe("ProgressBar", () => {
  it("renders correct width for given percent", () => {
    const { container } = render(<ProgressBar percent={60} />);
    const bar = container.querySelector("[style]");
    expect(bar).toHaveStyle({ width: "60%" });
  });

  it("displays rounded percentage text", () => {
    render(<ProgressBar percent={66.7} />);
    expect(screen.getByText("67%")).toBeInTheDocument();
  });

  it("clamps values above 100", () => {
    const { container } = render(<ProgressBar percent={150} />);
    const bar = container.querySelector("[style]");
    expect(bar).toHaveStyle({ width: "100%" });
    expect(screen.getByText("100%")).toBeInTheDocument();
  });

  it("clamps negative values to 0", () => {
    const { container } = render(<ProgressBar percent={-10} />);
    const bar = container.querySelector("[style]");
    expect(bar).toHaveStyle({ width: "0%" });
    expect(screen.getByText("0%")).toBeInTheDocument();
  });

  it("shows green color for >= 75%", () => {
    const { container } = render(<ProgressBar percent={80} />);
    const bar = container.querySelector("[style]");
    expect(bar?.className).toContain("bg-green-500");
  });

  it("shows blue color for >= 50%", () => {
    const { container } = render(<ProgressBar percent={60} />);
    const bar = container.querySelector("[style]");
    expect(bar?.className).toContain("bg-blue-500");
  });

  it("shows amber color for >= 25%", () => {
    const { container } = render(<ProgressBar percent={30} />);
    const bar = container.querySelector("[style]");
    expect(bar?.className).toContain("bg-amber-500");
  });

  it("shows red color for < 25%", () => {
    const { container } = render(<ProgressBar percent={10} />);
    const bar = container.querySelector("[style]");
    expect(bar?.className).toContain("bg-red-500");
  });
});
