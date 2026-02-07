"use client";

import { UpcomingReleases } from "./upcoming-releases";
import { RecentReleases } from "./recent-releases";
import { ReleaseTimeline } from "./release-timeline";
import type { ReleasePlan } from "@/types/api";

interface ReleasePlanViewProps {
  data: ReleasePlan;
}

export function ReleasePlanView({ data }: ReleasePlanViewProps) {
  return (
    <div className="space-y-8">
      {/* Upcoming Releases */}
      <section>
        <h3 className="mb-3 text-lg font-semibold text-gray-900 dark:text-white">
          Upcoming Releases
        </h3>
        <UpcomingReleases items={data.upcoming} />
      </section>

      {/* Recent Releases */}
      <section>
        <h3 className="mb-3 text-lg font-semibold text-gray-900 dark:text-white">
          Recent Releases
        </h3>
        <RecentReleases items={data.recent_releases} />
      </section>

      {/* Release Timeline */}
      <section>
        <h3 className="mb-3 text-lg font-semibold text-gray-900 dark:text-white">
          Release Timeline
        </h3>
        <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-900">
          <ReleaseTimeline entries={data.timeline} />
        </div>
      </section>
    </div>
  );
}
