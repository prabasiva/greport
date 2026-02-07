"use client";

import { cn } from "@/lib/utils";

export interface FilterState {
  issues: boolean;
  milestones: boolean;
  releases: boolean;
  pulls: boolean;
}

interface CalendarFiltersProps {
  filters: FilterState;
  onToggle: (key: keyof FilterState) => void;
}

const filterConfig: { key: keyof FilterState; label: string; color: string }[] = [
  { key: "issues", label: "Issues", color: "bg-blue-500" },
  { key: "milestones", label: "Milestones", color: "bg-purple-500" },
  { key: "releases", label: "Releases", color: "bg-green-500" },
  { key: "pulls", label: "PRs", color: "bg-amber-500" },
];

export function CalendarFilters({ filters, onToggle }: CalendarFiltersProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {filterConfig.map(({ key, label, color }) => (
        <button
          key={key}
          onClick={() => onToggle(key)}
          className={cn(
            "flex items-center gap-1.5 rounded-full px-3 py-1 text-sm font-medium transition-colors",
            filters[key]
              ? "bg-gray-100 text-gray-900 dark:bg-gray-800 dark:text-white"
              : "bg-gray-50 text-gray-400 dark:bg-gray-900 dark:text-gray-500",
          )}
        >
          <span
            className={cn(
              "h-2.5 w-2.5 rounded-full",
              filters[key] ? color : "bg-gray-300 dark:bg-gray-600",
            )}
          />
          {label}
        </button>
      ))}
    </div>
  );
}
