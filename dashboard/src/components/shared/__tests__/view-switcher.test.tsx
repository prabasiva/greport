import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { ViewSwitcher } from "../view-switcher";

const views = [
  { key: "cards", label: "Cards" },
  { key: "table", label: "Table" },
  { key: "timeline", label: "Timeline" },
];

describe("ViewSwitcher", () => {
  it("renders all view options", () => {
    render(
      <ViewSwitcher views={views} activeView="cards" onViewChange={() => {}} />,
    );
    expect(screen.getByText("Cards")).toBeInTheDocument();
    expect(screen.getByText("Table")).toBeInTheDocument();
    expect(screen.getByText("Timeline")).toBeInTheDocument();
  });

  it("highlights the active view", () => {
    render(
      <ViewSwitcher views={views} activeView="table" onViewChange={() => {}} />,
    );
    const activeBtn = screen.getByText("Table");
    expect(activeBtn.className).toContain("bg-blue-600");
    expect(activeBtn.className).toContain("text-white");
  });

  it("does not highlight inactive views", () => {
    render(
      <ViewSwitcher views={views} activeView="cards" onViewChange={() => {}} />,
    );
    const inactiveBtn = screen.getByText("Table");
    expect(inactiveBtn.className).not.toContain("bg-blue-600");
  });

  it("calls onViewChange with the correct key when clicked", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    render(
      <ViewSwitcher views={views} activeView="cards" onViewChange={handler} />,
    );
    await user.click(screen.getByText("Timeline"));
    expect(handler).toHaveBeenCalledWith("timeline");
  });

  it("calls onViewChange only once per click", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    render(
      <ViewSwitcher views={views} activeView="cards" onViewChange={handler} />,
    );
    await user.click(screen.getByText("Table"));
    expect(handler).toHaveBeenCalledTimes(1);
  });
});
