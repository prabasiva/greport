"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useRepos } from "@/hooks/use-repos";
import { useReleases, useReleaseNotes } from "@/hooks/use-api";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatDate, formatRelativeTime } from "@/lib/utils";
import type { Release } from "@/types/api";

export default function ReleasesPage() {
  const { activeRepo, repos } = useRepos();

  if (!activeRepo) {
    return <AggregateReleasesView repos={repos} />;
  }

  return <ReleasesContent owner={activeRepo.owner} repo={activeRepo.name} />;
}

function AggregateReleasesView({ repos }: { repos: { owner: string; name: string; fullName: string }[] }) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Releases
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          All repositories
        </p>
      </div>
      {repos.length === 0 ? (
        <div className="py-12 text-center text-sm text-gray-500">
          No repositories tracked. Add a repository to see releases.
        </div>
      ) : (
        repos.map((repo) => (
          <RepoReleasesSection key={repo.fullName} owner={repo.owner} name={repo.name} fullName={repo.fullName} />
        ))
      )}
    </div>
  );
}

function RepoReleasesSection({ owner, name, fullName }: { owner: string; name: string; fullName: string }) {
  const { data, error, isLoading } = useReleases(owner, name, { per_page: 5 });

  if (isLoading) return null;
  if (error) return null;

  const releases = data?.data || [];
  if (releases.length === 0) return null;

  return (
    <div className="space-y-3">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2">
        {fullName}
        <span className="ml-2 text-sm font-normal text-gray-500">
          {releases.length} release{releases.length !== 1 ? "s" : ""}
        </span>
      </h3>
      {releases.map((release, index) => (
        <ReleaseCard key={release.id} release={release} isLatest={index === 0} />
      ))}
    </div>
  );
}

function ReleasesContent({ owner, repo }: { owner: string; repo: string }) {
  const { data, error, isLoading } = useReleases(owner, repo, { per_page: 20 });
  const [selectedMilestone, setSelectedMilestone] = useState<string | null>(null);

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const releases = data?.data || [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Releases
          </h2>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            {owner}/{repo}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="Milestone name..."
            className="rounded-md border border-gray-300 px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
            value={selectedMilestone || ""}
            onChange={(e) => setSelectedMilestone(e.target.value || null)}
          />
          {selectedMilestone && (
            <button
              onClick={() => setSelectedMilestone(null)}
              className="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400"
            >
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Release Notes Preview */}
      {selectedMilestone && (
        <ReleaseNotesSection owner={owner} repo={repo} milestone={selectedMilestone} />
      )}

      {/* Release Timeline */}
      {releases.length === 0 ? (
        <div className="py-12 text-center text-sm text-gray-500">
          No releases found
        </div>
      ) : (
        <div className="space-y-4">
          {releases.map((release, index) => (
            <ReleaseCard key={release.id} release={release} isLatest={index === 0} />
          ))}
        </div>
      )}
    </div>
  );
}

function ReleaseCard({ release, isLatest }: { release: Release; isLatest: boolean }) {
  const [expanded, setExpanded] = useState(isLatest);

  return (
    <div className="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-800 dark:bg-gray-900">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center justify-between p-4 text-left"
      >
        <div className="flex items-center gap-3">
          <div className={`h-3 w-3 rounded-full ${release.prerelease ? "bg-amber-400" : release.draft ? "bg-gray-400" : "bg-green-500"}`} />
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-gray-900 dark:text-white">
                {release.name || release.tag_name}
              </h3>
              <span className="rounded bg-gray-100 px-2 py-0.5 text-xs font-mono text-gray-600 dark:bg-gray-800 dark:text-gray-400">
                {release.tag_name}
              </span>
              {isLatest && (
                <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-700 dark:bg-green-900 dark:text-green-300">
                  Latest
                </span>
              )}
              {release.prerelease && (
                <span className="rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-700 dark:bg-amber-900 dark:text-amber-300">
                  Pre-release
                </span>
              )}
              {release.draft && (
                <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-500 dark:bg-gray-800">
                  Draft
                </span>
              )}
            </div>
            <p className="mt-0.5 text-sm text-gray-500 dark:text-gray-400">
              Published {release.published_at ? formatRelativeTime(release.published_at) : formatRelativeTime(release.created_at)}{" "}
              by {release.author.login}
            </p>
          </div>
        </div>
        <svg
          className={`h-5 w-5 text-gray-400 transition-transform ${expanded ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
        </svg>
      </button>
      {expanded && release.body && (
        <div className="border-t border-gray-200 p-4 dark:border-gray-800">
          <pre className="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300">
            {release.body}
          </pre>
        </div>
      )}
    </div>
  );
}

function ReleaseNotesSection({
  owner,
  repo,
  milestone,
}: {
  owner: string;
  repo: string;
  milestone: string;
}) {
  const { data, error, isLoading } = useReleaseNotes(owner, repo, milestone);

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay title="Release Notes Error" message={error.message} />;

  const notes = data?.data;
  if (!notes) return null;

  return (
    <div className="rounded-lg border border-blue-200 bg-blue-50 p-6 dark:border-blue-900 dark:bg-blue-950">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
        Release Notes: {notes.version}
      </h3>
      <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
        {formatDate(notes.date)}
      </p>

      {notes.summary && (
        <p className="mt-3 text-sm text-gray-700 dark:text-gray-300">
          {notes.summary}
        </p>
      )}

      {notes.sections.map((section) => (
        <div key={section.title} className="mt-4">
          <h4 className="font-medium text-gray-900 dark:text-white">
            {section.title}
          </h4>
          <ul className="mt-2 space-y-1">
            {section.items.map((item) => (
              <li key={item.number} className="text-sm text-gray-700 dark:text-gray-300">
                #{item.number} - {item.title}{" "}
                <span className="text-gray-400">by {item.author}</span>
              </li>
            ))}
          </ul>
        </div>
      ))}

      <div className="mt-4 flex gap-4 text-sm text-gray-500 dark:text-gray-400">
        <span>{notes.stats.issues_closed} issues closed</span>
        <span>{notes.stats.prs_merged} PRs merged</span>
        <span>{notes.stats.contributors_count} contributors</span>
      </div>

      {notes.contributors.length > 0 && (
        <div className="mt-3">
          <span className="text-sm text-gray-500 dark:text-gray-400">
            Contributors: {notes.contributors.join(", ")}
          </span>
        </div>
      )}
    </div>
  );
}
