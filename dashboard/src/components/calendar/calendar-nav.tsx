"use client";

import type { CalendarViewMode } from "@/hooks/use-settings";
import { cn } from "@/lib/utils";

interface CalendarNavProps {
  baseMonth: Date;
  viewMode: CalendarViewMode;
  onPrev: () => void;
  onNext: () => void;
  onToday: () => void;
  onViewModeChange: (mode: CalendarViewMode) => void;
}

const MONTH_NAMES = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

export function CalendarNav({
  baseMonth,
  viewMode,
  onPrev,
  onNext,
  onToday,
  onViewModeChange,
}: CalendarNavProps) {
  const months: string[] = [];
  if (viewMode === "1") {
    months.push(
      `${MONTH_NAMES[baseMonth.getMonth()]} ${baseMonth.getFullYear()}`,
    );
  } else {
    for (let i = -1; i <= 1; i++) {
      const d = new Date(baseMonth.getFullYear(), baseMonth.getMonth() + i, 1);
      months.push(`${MONTH_NAMES[d.getMonth()]} ${d.getFullYear()}`);
    }
  }

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        <button
          onClick={onPrev}
          className="rounded-md border border-gray-300 p-1.5 text-gray-500 hover:bg-gray-50 dark:border-gray-600 dark:text-gray-400 dark:hover:bg-gray-800"
          aria-label="Previous month"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
          </svg>
        </button>
        <button
          onClick={onToday}
          className="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-800"
        >
          Today
        </button>
        <button
          onClick={onNext}
          className="rounded-md border border-gray-300 p-1.5 text-gray-500 hover:bg-gray-50 dark:border-gray-600 dark:text-gray-400 dark:hover:bg-gray-800"
          aria-label="Next month"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="m8.25 4.5 7.5 7.5-7.5 7.5" />
          </svg>
        </button>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex gap-4 text-sm font-medium text-gray-700 dark:text-gray-300">
          {months.map((label) => (
            <span key={label}>{label}</span>
          ))}
        </div>

        <div className="flex rounded-md border border-gray-300 dark:border-gray-600">
          <button
            onClick={() => onViewModeChange("1")}
            className={cn(
              "px-2.5 py-1 text-xs font-medium transition-colors",
              viewMode === "1"
                ? "bg-blue-600 text-white"
                : "text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-800",
              "rounded-l-md",
            )}
          >
            1M
          </button>
          <button
            onClick={() => onViewModeChange("3")}
            className={cn(
              "px-2.5 py-1 text-xs font-medium transition-colors",
              viewMode === "3"
                ? "bg-blue-600 text-white"
                : "text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-800",
              "rounded-r-md border-l border-gray-300 dark:border-gray-600",
            )}
          >
            3M
          </button>
        </div>
      </div>
    </div>
  );
}
