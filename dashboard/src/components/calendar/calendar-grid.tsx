"use client";

import { CalendarDay } from "./calendar-day";
import type { CalendarEvent } from "@/types/api";

interface CalendarGridProps {
  year: number;
  month: number;
  events: CalendarEvent[];
  onSelectDay: (events: CalendarEvent[]) => void;
  expanded?: boolean;
}

const DAY_NAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

export function CalendarGrid({ year, month, events, onSelectDay, expanded }: CalendarGridProps) {
  const firstDay = new Date(year, month, 1);
  const startDow = firstDay.getDay(); // 0=Sun
  const daysInMonth = new Date(year, month + 1, 0).getDate();

  // Previous month days to fill first row
  const prevMonthDays = new Date(year, month, 0).getDate();
  const today = new Date();
  const isCurrentMonth =
    today.getFullYear() === year && today.getMonth() === month;

  // Build event lookup: day number -> events[]
  const eventsByDay: Record<number, CalendarEvent[]> = {};
  for (const event of events) {
    const d = new Date(event.date);
    if (d.getFullYear() === year && d.getMonth() === month) {
      const day = d.getDate();
      if (!eventsByDay[day]) eventsByDay[day] = [];
      eventsByDay[day].push(event);
    }
  }

  // Build cells
  const cells: { day: number; isCurrentMonth: boolean; isToday: boolean; events: CalendarEvent[] }[] = [];

  // Previous month trailing days
  for (let i = startDow - 1; i >= 0; i--) {
    cells.push({
      day: prevMonthDays - i,
      isCurrentMonth: false,
      isToday: false,
      events: [],
    });
  }

  // Current month days
  for (let d = 1; d <= daysInMonth; d++) {
    cells.push({
      day: d,
      isCurrentMonth: true,
      isToday: isCurrentMonth && today.getDate() === d,
      events: eventsByDay[d] || [],
    });
  }

  // Next month leading days to fill the grid
  const remaining = 7 - (cells.length % 7);
  if (remaining < 7) {
    for (let i = 1; i <= remaining; i++) {
      cells.push({
        day: i,
        isCurrentMonth: false,
        isToday: false,
        events: [],
      });
    }
  }

  return (
    <div>
      <h3
        className={
          expanded
            ? "mb-3 text-lg font-semibold text-gray-900 dark:text-white"
            : "mb-2 text-sm font-semibold text-gray-900 dark:text-white"
        }
      >
        {MONTH_NAMES[month]} {year}
      </h3>
      <div className="grid grid-cols-7 gap-px">
        {DAY_NAMES.map((name) => (
          <div
            key={name}
            className={
              expanded
                ? "py-2 text-center text-sm font-medium text-gray-500 dark:text-gray-400"
                : "py-1 text-center text-xs font-medium text-gray-500 dark:text-gray-400"
            }
          >
            {name}
          </div>
        ))}
        {cells.map((cell, i) => (
          <CalendarDay
            key={i}
            day={cell.day}
            isCurrentMonth={cell.isCurrentMonth}
            isToday={cell.isToday}
            events={cell.events}
            onSelect={onSelectDay}
            expanded={expanded}
          />
        ))}
      </div>
    </div>
  );
}
