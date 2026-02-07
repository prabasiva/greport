"use client";

import type { CalendarEvent, CalendarEventType } from "@/types/api";
import { cn } from "@/lib/utils";

interface EventPopoverProps {
  events: CalendarEvent[];
  onClose: () => void;
}

const eventTypeLabels: Record<CalendarEventType, string> = {
  issue_created: "Issue Created",
  issue_closed: "Issue Closed",
  milestone_due: "Milestone Due",
  milestone_closed: "Milestone Closed",
  release_published: "Release Published",
  pr_merged: "PR Merged",
};

const eventTypeColors: Record<CalendarEventType, string> = {
  issue_created: "text-blue-500",
  issue_closed: "text-blue-700 dark:text-blue-400",
  milestone_due: "text-purple-500",
  milestone_closed: "text-purple-700 dark:text-purple-400",
  release_published: "text-green-600 dark:text-green-400",
  pr_merged: "text-amber-600 dark:text-amber-400",
};

const eventTypeIcons: Record<CalendarEventType, string> = {
  issue_created: "M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
  issue_closed: "M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
  milestone_due: "M3 3v1.5M3 21v-6m0 0 2.77-.693a9 9 0 0 1 6.208.682l.108.054a9 9 0 0 0 6.086.71l3.114-.732a48.524 48.524 0 0 1-.005-10.499l-3.11.732a9 9 0 0 1-6.085-.711l-.108-.054a9 9 0 0 0-6.208-.682L3 4.5M3 15V4.5",
  milestone_closed: "M3 3v1.5M3 21v-6m0 0 2.77-.693a9 9 0 0 1 6.208.682l.108.054a9 9 0 0 0 6.086.71l3.114-.732a48.524 48.524 0 0 1-.005-10.499l-3.11.732a9 9 0 0 1-6.085-.711l-.108-.054a9 9 0 0 0-6.208-.682L3 4.5M3 15V4.5",
  release_published: "M9.568 3H5.25A2.25 2.25 0 0 0 3 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 0 0 5.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 0 0 9.568 3Z",
  pr_merged: "M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5",
};

export function EventPopover({ events, onClose }: EventPopoverProps) {
  if (events.length === 0) return null;

  const dateStr = new Date(events[0].date).toLocaleDateString("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="max-h-[80vh] w-full max-w-lg overflow-y-auto rounded-lg border border-gray-200 bg-white p-6 shadow-xl dark:border-gray-700 dark:bg-gray-900"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            {dateStr}
          </h3>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-800 dark:hover:text-gray-300"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <p className="mb-4 text-sm text-gray-500 dark:text-gray-400">
          {events.length} event{events.length !== 1 ? "s" : ""}
        </p>
        <ul className="space-y-3">
          {events.map((event) => (
            <li key={event.id} className="flex gap-3 rounded-md p-2 hover:bg-gray-50 dark:hover:bg-gray-800">
              <svg
                className={cn("mt-0.5 h-5 w-5 shrink-0", eventTypeColors[event.event_type])}
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth={1.5}
                stroke="currentColor"
              >
                <path strokeLinecap="round" strokeLinejoin="round" d={eventTypeIcons[event.event_type]} />
              </svg>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <a
                    href={event.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="truncate text-sm font-medium text-gray-900 hover:text-blue-600 dark:text-white dark:hover:text-blue-400"
                  >
                    {event.number ? `#${event.number} ` : ""}
                    {event.title}
                  </a>
                </div>
                <div className="mt-0.5 flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                  <span className={eventTypeColors[event.event_type]}>
                    {eventTypeLabels[event.event_type]}
                  </span>
                  <span>{event.repository}</span>
                </div>
                {event.labels.length > 0 && (
                  <div className="mt-1 flex flex-wrap gap-1">
                    {event.labels.map((label) => (
                      <span
                        key={label}
                        className="rounded bg-gray-100 px-1.5 py-0.5 text-[10px] text-gray-600 dark:bg-gray-800 dark:text-gray-400"
                      >
                        {label}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
