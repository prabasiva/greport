"use client";

import { cn } from "@/lib/utils";
import { ProgressBar } from "./progress-bar";
import type { UpcomingRelease, ReleasePlanStatus } from "@/types/api";

const statusConfig: Record<ReleasePlanStatus, { label: string; class: string }> = {
  on_track: {
    label: "On Track",
    class: "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  },
  at_risk: {
    label: "At Risk",
    class: "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
  },
  overdue: {
    label: "Overdue",
    class: "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300",
  },
};

interface MilestoneCardProps {
  item: UpcomingRelease;
}

export function MilestoneCard({ item }: MilestoneCardProps) {
  const { milestone, repository, progress_percent, days_remaining, blocker_count, status } = item;
  const statusInfo = statusConfig[status];

  const dueLabel = milestone.due_on
    ? new Date(milestone.due_on).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
      })
    : "No due date";

  const daysLabel =
    days_remaining > 0
      ? `${days_remaining}d remaining`
      : days_remaining === 0
        ? "Due today"
        : `${Math.abs(days_remaining)}d overdue`;

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-900">
      <div className="flex items-start justify-between">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h4 className="truncate text-sm font-semibold text-gray-900 dark:text-white">
              {milestone.title}
            </h4>
            <span className={cn("shrink-0 rounded-full px-2 py-0.5 text-xs font-medium", statusInfo.class)}>
              {statusInfo.label}
            </span>
          </div>
          <p className="mt-0.5 text-xs text-gray-500 dark:text-gray-400">{repository}</p>
        </div>
      </div>

      <ProgressBar percent={progress_percent} className="mt-3" />

      <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-gray-500 dark:text-gray-400">
        <span>Due: {dueLabel}</span>
        <span className={days_remaining < 0 ? "font-medium text-red-600 dark:text-red-400" : ""}>
          {daysLabel}
        </span>
        <span>{milestone.closed_issues} closed / {milestone.open_issues} open</span>
        {blocker_count > 0 && (
          <span className="font-medium text-red-600 dark:text-red-400">
            {blocker_count} blocker{blocker_count !== 1 ? "s" : ""}
          </span>
        )}
      </div>
    </div>
  );
}
