import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { CalendarGrid } from "../calendar-grid";
import { createCalendarEvent } from "@/test/factories";

describe("CalendarGrid", () => {
  it("renders month name and year heading", () => {
    render(
      <CalendarGrid
        year={2025}
        month={0}
        events={[]}
        onSelectDay={vi.fn()}
      />,
    );
    expect(screen.getByText("January 2025")).toBeInTheDocument();
  });

  it("renders day-of-week headers", () => {
    render(
      <CalendarGrid
        year={2025}
        month={3}
        events={[]}
        onSelectDay={vi.fn()}
      />,
    );
    expect(screen.getByText("Sun")).toBeInTheDocument();
    expect(screen.getByText("Mon")).toBeInTheDocument();
    expect(screen.getByText("Tue")).toBeInTheDocument();
    expect(screen.getByText("Wed")).toBeInTheDocument();
    expect(screen.getByText("Thu")).toBeInTheDocument();
    expect(screen.getByText("Fri")).toBeInTheDocument();
    expect(screen.getByText("Sat")).toBeInTheDocument();
  });

  it("renders the correct number of day cells (full weeks)", () => {
    const { container } = render(
      <CalendarGrid
        year={2025}
        month={1}
        events={[]}
        onSelectDay={vi.fn()}
      />,
    );
    // February 2025 starts on Saturday (index 6), 28 days
    // 6 prev-month days + 28 current days = 34, pad to 35 (5 rows)
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(35);
  });

  it("places events on the correct day", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        title: "Bug fix",
        date: "2025-06-15T12:00:00Z",
        event_type: "issue_closed",
      }),
    ];
    const { container } = render(
      <CalendarGrid
        year={2025}
        month={5}
        events={events}
        onSelectDay={vi.fn()}
      />,
    );
    // The event dot should be present somewhere in the grid
    const dots = container.querySelectorAll(".rounded-full.h-1\\.5");
    expect(dots.length).toBe(1);
  });

  it("renders events in expanded mode with titles", () => {
    const events = [
      createCalendarEvent({
        id: "e1",
        title: "Deploy service",
        number: 99,
        date: "2025-06-10T12:00:00Z",
        event_type: "release_published",
      }),
    ];
    render(
      <CalendarGrid
        year={2025}
        month={5}
        events={events}
        onSelectDay={vi.fn()}
        expanded={true}
      />,
    );
    expect(screen.getByText("#99 Deploy service")).toBeInTheDocument();
  });
});
