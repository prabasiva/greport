"use client";

import { cn } from "@/lib/utils";
import type { RecentRelease } from "@/types/api";

const typeConfig: Record<string, { label: string; class: string }> = {
  stable: {
    label: "Stable",
    class: "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  },
  prerelease: {
    label: "Pre-release",
    class: "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
  },
  draft: {
    label: "Draft",
    class: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
  },
};

interface ReleaseCardProps {
  item: RecentRelease;
}

export function ReleaseCard({ item }: ReleaseCardProps) {
  const { release, repository, release_type } = item;
  const typeInfo = typeConfig[release_type] || typeConfig.stable;

  const publishedLabel = release.published_at
    ? new Date(release.published_at).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
      })
    : "Not published";

  const displayName = release.name || release.tag_name;

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-900">
      <div className="flex items-start justify-between">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h4 className="truncate text-sm font-semibold text-gray-900 dark:text-white">
              {displayName}
            </h4>
            <span className={cn("shrink-0 rounded-full px-2 py-0.5 text-xs font-medium", typeInfo.class)}>
              {typeInfo.label}
            </span>
          </div>
          <p className="mt-0.5 text-xs text-gray-500 dark:text-gray-400">{repository}</p>
        </div>
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-gray-500 dark:text-gray-400">
        <span>Published: {publishedLabel}</span>
        <span>Tag: {release.tag_name}</span>
        {release.author && <span>By: {release.author.login}</span>}
      </div>

      {release.body && (
        <p className="mt-2 line-clamp-2 text-xs text-gray-600 dark:text-gray-300">
          {release.body}
        </p>
      )}
    </div>
  );
}
