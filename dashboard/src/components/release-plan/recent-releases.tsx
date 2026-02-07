"use client";

import { ReleaseCard } from "./release-card";
import type { RecentRelease } from "@/types/api";

interface RecentReleasesProps {
  items: RecentRelease[];
}

export function RecentReleases({ items }: RecentReleasesProps) {
  if (items.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-gray-300 p-6 text-center dark:border-gray-700">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          No releases published in the selected time range.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {items.map((item) => (
        <ReleaseCard
          key={`${item.repository}-${item.release.id}`}
          item={item}
        />
      ))}
    </div>
  );
}
