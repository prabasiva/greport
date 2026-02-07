"use client";

import { MilestoneCard } from "./milestone-card";
import type { UpcomingRelease } from "@/types/api";

interface UpcomingReleasesProps {
  items: UpcomingRelease[];
}

export function UpcomingReleases({ items }: UpcomingReleasesProps) {
  if (items.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-gray-300 p-6 text-center dark:border-gray-700">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          No upcoming milestones with due dates found.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {items.map((item) => (
        <MilestoneCard key={`${item.repository}-${item.milestone.id}`} item={item} />
      ))}
    </div>
  );
}
