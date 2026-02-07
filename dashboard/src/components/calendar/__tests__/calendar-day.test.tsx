import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { CalendarDay } from "../calendar-day";
import { createCalendarEvent } from "@/test/factories";

describe("CalendarDay", () => {
  const defaultProps = {
    day: 15,
    isCurrentMonth: true,
    isToday: false,
    events: [] as ReturnType<typeof createCalendarEvent>[],
    onSelect: vi.fn(),
  };

  it("renders the day number", () => {
    render(<CalendarDay {...defaultProps} />);
    expect(screen.getByText("15")).toBeInTheDocument();
  });

  it("renders event dots for events", () => {
    const events = [
      createCalendarEvent({ id: "e1", event_type: "issue_created" }),
      createCalendarEvent({ id: "e2", event_type: "release_published" }),
    ];
    const { container } = render(
      <CalendarDay {...defaultProps} events={events} />,
    );
    const dots = container.querySelectorAll(".rounded-full.h-1\\.5");
    expect(dots.length).toBe(2);
  });

  it("shows +N when more than 3 events in compact mode", () => {
    const events = [
      createCalendarEvent({ id: "e1" }),
      createCalendarEvent({ id: "e2" }),
      createCalendarEvent({ id: "e3" }),
      createCalendarEvent({ id: "e4" }),
      createCalendarEvent({ id: "e5" }),
    ];
    render(<CalendarDay {...defaultProps} events={events} />);
    expect(screen.getByText("+2")).toBeInTheDocument();
  });

  it("calls onSelect with events when clicked", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    const events = [createCalendarEvent({ id: "e1" })];
    render(
      <CalendarDay {...defaultProps} events={events} onSelect={handler} />,
    );
    await user.click(screen.getByRole("button"));
    expect(handler).toHaveBeenCalledWith(events);
  });

  it("does not call onSelect when there are no events", async () => {
    const user = userEvent.setup();
    const handler = vi.fn();
    render(<CalendarDay {...defaultProps} onSelect={handler} />);
    await user.click(screen.getByRole("button"));
    expect(handler).not.toHaveBeenCalled();
  });

  it("renders expanded mode with event titles", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        title: "Fix login bug",
        number: 42,
        event_type: "issue_created",
      }),
    ];
    render(
      <CalendarDay {...defaultProps} events={events} expanded={true} />,
    );
    expect(screen.getByText("#42 Fix login bug")).toBeInTheDocument();
  });

  it("shows +N more in expanded mode when over 4 events", () => {
    const events = Array.from({ length: 6 }, (_, i) =>
      createCalendarEvent({ id: `e${i}`, title: `Event ${i}` }),
    );
    render(
      <CalendarDay {...defaultProps} events={events} expanded={true} />,
    );
    expect(screen.getByText("+2 more")).toBeInTheDocument();
  });

  it("highlights today with blue background", () => {
    const { container } = render(
      <CalendarDay {...defaultProps} isToday={true} />,
    );
    const daySpan = container.querySelector(".bg-blue-600");
    expect(daySpan).toBeInTheDocument();
  });
});
