import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { EventPopover } from "../event-popover";
import { createCalendarEvent } from "@/test/factories";

describe("EventPopover", () => {
  it("renders nothing when events list is empty", () => {
    const { container } = render(
      <EventPopover events={[]} onClose={vi.fn()} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders the formatted date heading", () => {
    const events = [
      createCalendarEvent({ date: "2025-06-15T12:00:00Z" }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    // Should show the date formatted like "Sunday, June 15, 2025"
    expect(screen.getByText(/June 15, 2025/)).toBeInTheDocument();
  });

  it("renders event count text", () => {
    const events = [
      createCalendarEvent({ id: "e1" }),
      createCalendarEvent({ id: "e2" }),
      createCalendarEvent({ id: "e3" }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("3 events")).toBeInTheDocument();
  });

  it("renders singular event text for one event", () => {
    const events = [createCalendarEvent({ id: "e1" })];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("1 event")).toBeInTheDocument();
  });

  it("renders event titles with issue numbers", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        title: "Fix auth",
        number: 42,
        event_type: "issue_closed",
      }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("#42 Fix auth")).toBeInTheDocument();
  });

  it("renders event type labels", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        event_type: "release_published",
      }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("Release Published")).toBeInTheDocument();
  });

  it("renders repository name for each event", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        repository: "org/my-app",
      }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("org/my-app")).toBeInTheDocument();
  });

  it("renders labels as badges", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        labels: ["bug", "critical"],
      }),
    ];
    render(<EventPopover events={events} onClose={vi.fn()} />);
    expect(screen.getByText("bug")).toBeInTheDocument();
    expect(screen.getByText("critical")).toBeInTheDocument();
  });

  it("calls onClose when backdrop is clicked", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    const events = [createCalendarEvent({ id: "e1" })];
    const { container } = render(
      <EventPopover events={events} onClose={handler} />,
    );
    // Click on the backdrop (the fixed overlay)
    const backdrop = container.querySelector(".fixed.inset-0");
    if (backdrop) {
      await user.click(backdrop);
    }
    expect(handler).toHaveBeenCalled();
  });

  it("calls onClose when close button is clicked", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    const events = [createCalendarEvent({ id: "e1" })];
    render(<EventPopover events={events} onClose={handler} />);
    // The close button contains an X icon SVG
    const buttons = screen.getAllByRole("button");
    // Close button is the one inside the popover
    await user.click(buttons[0]);
    expect(handler).toHaveBeenCalled();
  });
});
