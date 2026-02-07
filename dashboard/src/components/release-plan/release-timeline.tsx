"use client";

import { useMemo } from "react";
import { cn } from "@/lib/utils";
import type { TimelineEntry } from "@/types/api";

interface ReleaseTimelineProps {
  entries: TimelineEntry[];
}

const repoColors = [
  "bg-blue-500",
  "bg-green-500",
  "bg-purple-500",
  "bg-amber-500",
  "bg-rose-500",
];

const repoBorderColors = [
  "border-blue-500",
  "border-green-500",
  "border-purple-500",
  "border-amber-500",
  "border-rose-500",
];

export function ReleaseTimeline({ entries }: ReleaseTimelineProps) {
  const { months, repoColorMap, positioned } = useMemo(() => {
    if (entries.length === 0) {
      return { months: [] as string[], repoColorMap: {} as Record<string, number>, positioned: [] as (TimelineEntry & { offset: number })[] };
    }

    // Determine date range
    const dates = entries.map((e) => new Date(e.date).getTime());
    const minDate = new Date(Math.min(...dates));
    const maxDate = new Date(Math.max(...dates));

    // Build month labels
    const monthLabels: string[] = [];
    const cur = new Date(minDate.getFullYear(), minDate.getMonth(), 1);
    const endMonth = new Date(maxDate.getFullYear(), maxDate.getMonth() + 1, 1);
    while (cur < endMonth) {
      monthLabels.push(
        cur.toLocaleDateString("en-US", { month: "short", year: "numeric" }),
      );
      cur.setMonth(cur.getMonth() + 1);
    }

    // Build repo color map
    const repos = [...new Set(entries.map((e) => e.repository))];
    const colorMap: Record<string, number> = {};
    repos.forEach((r, i) => {
      colorMap[r] = i % repoColors.length;
    });

    // Position entries as percentage along the timeline
    const totalRange = maxDate.getTime() - minDate.getTime() || 1;
    const pos = entries.map((e) => ({
      ...e,
      offset: ((new Date(e.date).getTime() - minDate.getTime()) / totalRange) * 100,
    }));

    return { months: monthLabels, repoColorMap: colorMap, positioned: pos };
  }, [entries]);

  if (entries.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-gray-300 p-6 text-center dark:border-gray-700">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          No timeline data available.
        </p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      {/* Month labels */}
      <div className="flex justify-between px-2 text-xs text-gray-500 dark:text-gray-400">
        {months.map((m) => (
          <span key={m}>{m}</span>
        ))}
      </div>

      {/* Timeline bar */}
      <div className="relative mt-2 h-12 rounded-full bg-gray-100 dark:bg-gray-800">
        {/* Center line */}
        <div className="absolute left-0 right-0 top-1/2 h-0.5 -translate-y-1/2 bg-gray-300 dark:bg-gray-600" />

        {/* Entry markers */}
        {positioned.map((entry, i) => {
          const colorIdx = repoColorMap[entry.repository] ?? 0;
          const isMilestone = entry.entry_type === "milestone";
          return (
            <div
              key={`${entry.repository}-${entry.title}-${i}`}
              className="group absolute top-1/2 -translate-x-1/2 -translate-y-1/2"
              style={{ left: `${Math.min(98, Math.max(2, entry.offset))}%` }}
            >
              <div
                className={cn(
                  "h-4 w-4 rounded-full border-2",
                  isMilestone
                    ? `bg-transparent ${repoBorderColors[colorIdx]}`
                    : `${repoColors[colorIdx]} border-transparent`,
                )}
              />
              {/* Tooltip */}
              <div className="pointer-events-none absolute bottom-full left-1/2 mb-2 -translate-x-1/2 whitespace-nowrap rounded bg-gray-900 px-2 py-1 text-xs text-white opacity-0 transition-opacity group-hover:opacity-100 dark:bg-gray-700">
                <div className="font-medium">{entry.title}</div>
                <div className="text-gray-300">{entry.repository}</div>
                <div className="text-gray-400">
                  {new Date(entry.date).toLocaleDateString("en-US", {
                    month: "short",
                    day: "numeric",
                  })}
                  {entry.progress_percent != null && ` - ${Math.round(entry.progress_percent)}%`}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Legend */}
      <div className="mt-3 flex flex-wrap gap-3 text-xs text-gray-500 dark:text-gray-400">
        {Object.entries(repoColorMap).map(([repo, idx]) => (
          <div key={repo} className="flex items-center gap-1.5">
            <div className={cn("h-2.5 w-2.5 rounded-full", repoColors[idx])} />
            <span>{repo}</span>
          </div>
        ))}
        <div className="flex items-center gap-1.5">
          <div className="h-2.5 w-2.5 rounded-full border-2 border-gray-400 bg-transparent" />
          <span>Milestone</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-2.5 w-2.5 rounded-full bg-gray-400" />
          <span>Release</span>
        </div>
      </div>
    </div>
  );
}
