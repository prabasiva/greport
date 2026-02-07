"use client";

import { cn } from "@/lib/utils";
import type { CalendarEvent, CalendarEventType } from "@/types/api";

interface CalendarDayProps {
  day: number;
  isCurrentMonth: boolean;
  isToday: boolean;
  events: CalendarEvent[];
  onSelect: (events: CalendarEvent[]) => void;
  expanded?: boolean;
}

const eventTypeColors: Record<CalendarEventType, string> = {
  issue_created: "bg-blue-400",
  issue_closed: "bg-blue-600",
  milestone_due: "bg-purple-400",
  milestone_closed: "bg-purple-600",
  release_published: "bg-green-500",
  pr_merged: "bg-amber-500",
};

const eventTypeBgLight: Record<CalendarEventType, string> = {
  issue_created: "bg-blue-50 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
  issue_closed: "bg-blue-100 text-blue-800 dark:bg-blue-900/60 dark:text-blue-200",
  milestone_due: "bg-purple-50 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
  milestone_closed: "bg-purple-100 text-purple-800 dark:bg-purple-900/60 dark:text-purple-200",
  release_published: "bg-green-50 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  pr_merged: "bg-amber-50 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
};

export function CalendarDay({ day, isCurrentMonth, isToday, events, onSelect, expanded }: CalendarDayProps) {
  if (expanded) {
    return <ExpandedDay day={day} isCurrentMonth={isCurrentMonth} isToday={isToday} events={events} onSelect={onSelect} />;
  }

  const maxDots = 3;
  const visibleEvents = events.slice(0, maxDots);
  const extraCount = events.length - maxDots;

  return (
    <button
      onClick={() => events.length > 0 && onSelect(events)}
      className={cn(
        "flex min-h-[4.5rem] flex-col items-start rounded-md border p-1 text-left transition-colors",
        isCurrentMonth
          ? "border-gray-200 bg-white hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:hover:bg-gray-800"
          : "border-gray-100 bg-gray-50 dark:border-gray-800 dark:bg-gray-950",
        events.length > 0 && "cursor-pointer",
        events.length === 0 && "cursor-default",
      )}
    >
      <span
        className={cn(
          "mb-1 inline-flex h-6 w-6 items-center justify-center rounded-full text-xs font-medium",
          isToday
            ? "bg-blue-600 text-white"
            : isCurrentMonth
              ? "text-gray-900 dark:text-gray-100"
              : "text-gray-400 dark:text-gray-600",
        )}
      >
        {day}
      </span>
      <div className="flex flex-wrap gap-0.5">
        {visibleEvents.map((event) => (
          <span
            key={event.id}
            className={cn("h-1.5 w-1.5 rounded-full", eventTypeColors[event.event_type])}
            title={event.title}
          />
        ))}
        {extraCount > 0 && (
          <span className="text-[10px] leading-none text-gray-400">
            +{extraCount}
          </span>
        )}
      </div>
    </button>
  );
}

function ExpandedDay({
  day,
  isCurrentMonth,
  isToday,
  events,
  onSelect,
}: {
  day: number;
  isCurrentMonth: boolean;
  isToday: boolean;
  events: CalendarEvent[];
  onSelect: (events: CalendarEvent[]) => void;
}) {
  const maxVisible = 4;
  const visibleEvents = events.slice(0, maxVisible);
  const extraCount = events.length - maxVisible;

  return (
    <button
      onClick={() => events.length > 0 && onSelect(events)}
      className={cn(
        "flex min-h-[8rem] flex-col items-stretch rounded-lg border p-2 text-left transition-colors",
        isCurrentMonth
          ? "border-gray-200 bg-white hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-900 dark:hover:bg-gray-800"
          : "border-gray-100 bg-gray-50 dark:border-gray-800 dark:bg-gray-950",
        events.length > 0 && "cursor-pointer",
        events.length === 0 && "cursor-default",
      )}
    >
      <span
        className={cn(
          "mb-1.5 inline-flex h-7 w-7 items-center justify-center rounded-full text-sm font-semibold",
          isToday
            ? "bg-blue-600 text-white"
            : isCurrentMonth
              ? "text-gray-900 dark:text-gray-100"
              : "text-gray-400 dark:text-gray-600",
        )}
      >
        {day}
      </span>
      <div className="flex flex-1 flex-col gap-1 overflow-hidden">
        {visibleEvents.map((event) => (
          <div
            key={event.id}
            className={cn(
              "truncate rounded px-1.5 py-0.5 text-xs font-medium",
              eventTypeBgLight[event.event_type],
            )}
            title={event.title}
          >
            {event.number ? `#${event.number} ` : ""}
            {event.title}
          </div>
        ))}
        {extraCount > 0 && (
          <span className="px-1 text-xs text-gray-400 dark:text-gray-500">
            +{extraCount} more
          </span>
        )}
      </div>
    </button>
  );
}
