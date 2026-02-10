"use client";

import type { Issue, PullRequest, AggregateIssueItem, AggregatePullItem } from "@/types/api";
import { useOrgs } from "@/hooks/use-api";
import { formatDate, labelColor } from "@/lib/utils";

type DetailItem = Issue | PullRequest | AggregateIssueItem | AggregatePullItem;

interface DetailPopoverProps {
  item: DetailItem;
  owner: string;
  repo: string;
  onClose: () => void;
}

function isPullRequest(item: DetailItem): item is PullRequest | AggregatePullItem {
  return "merged" in item;
}

export function DetailPopover({ item, owner, repo, onClose }: DetailPopoverProps) {
  const { data: orgsData } = useOrgs();
  const isPr = isPullRequest(item);

  let webBase = "https://github.com";
  if (orgsData?.data) {
    const org = orgsData.data.orgs.find(o => o.name.toLowerCase() === owner.toLowerCase());
    webBase = org?.web_url ?? orgsData.data.default_web_url ?? "https://github.com";
  }
  const url = `${webBase}/${owner}/${repo}/${isPr ? "pull" : "issues"}/${item.number}`;

  // State badge
  let stateLabel: string;
  let stateCls: string;
  if (isPr && item.merged) {
    stateLabel = "merged";
    stateCls = "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300";
  } else if (item.state === "open") {
    stateLabel = "open";
    stateCls = "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300";
  } else {
    stateLabel = "closed";
    stateCls = isPr
      ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
      : "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300";
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded-lg border border-gray-200 bg-white p-6 shadow-xl dark:border-gray-700 dark:bg-gray-900"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="mb-4 flex items-start justify-between gap-4">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-mono text-sm text-gray-500">#{item.number}</span>
              <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${stateCls}`}>
                {stateLabel}
              </span>
              {isPr && item.draft && (
                <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500 dark:bg-gray-800 dark:text-gray-400">
                  Draft
                </span>
              )}
            </div>
            <h3 className="mt-1 text-lg font-semibold text-gray-900 dark:text-white">
              {item.title}
            </h3>
          </div>
          <button
            onClick={onClose}
            className="shrink-0 rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-800 dark:hover:text-gray-300"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Open on GitHub link */}
        <div className="mb-4">
          <a
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
          >
            Open on GitHub
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" />
            </svg>
          </a>
        </div>

        {/* Metadata grid */}
        <div className="mb-4 grid grid-cols-2 gap-x-6 gap-y-3 text-sm">
          {/* Author */}
          <div>
            <span className="text-gray-500 dark:text-gray-400">Author</span>
            <div className="mt-0.5 flex items-center gap-2">
              {item.author.avatar_url && (
                <img src={item.author.avatar_url} alt={item.author.login} className="h-5 w-5 rounded-full" />
              )}
              <span className="font-medium text-gray-900 dark:text-white">{item.author.login}</span>
            </div>
          </div>

          {/* Created */}
          <div>
            <span className="text-gray-500 dark:text-gray-400">Created</span>
            <p className="mt-0.5 font-medium text-gray-900 dark:text-white">{formatDate(item.created_at)}</p>
          </div>

          {/* Closed date */}
          {item.closed_at && (
            <div>
              <span className="text-gray-500 dark:text-gray-400">Closed</span>
              <p className="mt-0.5 font-medium text-gray-900 dark:text-white">{formatDate(item.closed_at)}</p>
            </div>
          )}

          {/* Merged date (PRs) */}
          {isPr && item.merged_at && (
            <div>
              <span className="text-gray-500 dark:text-gray-400">Merged</span>
              <p className="mt-0.5 font-medium text-gray-900 dark:text-white">{formatDate(item.merged_at)}</p>
            </div>
          )}

          {/* Branch refs (PRs) */}
          {isPr && (
            <div className="col-span-2">
              <span className="text-gray-500 dark:text-gray-400">Branches</span>
              <p className="mt-0.5 font-mono text-xs text-gray-900 dark:text-white">
                {item.head_ref} &rarr; {item.base_ref}
              </p>
            </div>
          )}

          {/* Changes (PRs) */}
          {isPr && (
            <div>
              <span className="text-gray-500 dark:text-gray-400">Changes</span>
              <p className="mt-0.5">
                <span className="font-medium text-green-600">+{item.additions}</span>
                {" / "}
                <span className="font-medium text-red-600">-{item.deletions}</span>
                <span className="ml-2 text-gray-500">({item.changed_files} files)</span>
              </p>
            </div>
          )}

          {/* Assignees (Issues) */}
          {!isPr && (item as Issue).assignees.length > 0 && (
            <div className="col-span-2">
              <span className="text-gray-500 dark:text-gray-400">Assignees</span>
              <div className="mt-0.5 flex flex-wrap gap-2">
                {(item as Issue).assignees.map((a) => (
                  <div key={a.id} className="flex items-center gap-1">
                    {a.avatar_url && (
                      <img src={a.avatar_url} alt={a.login} className="h-5 w-5 rounded-full" />
                    )}
                    <span className="font-medium text-gray-900 dark:text-white">{a.login}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Milestone */}
          {item.milestone && (
            <div>
              <span className="text-gray-500 dark:text-gray-400">Milestone</span>
              <p className="mt-0.5 font-medium text-gray-900 dark:text-white">{item.milestone.title}</p>
            </div>
          )}
        </div>

        {/* Labels */}
        {item.labels.length > 0 && (
          <div className="mb-4">
            <span className="text-sm text-gray-500 dark:text-gray-400">Labels</span>
            <div className="mt-1 flex flex-wrap gap-1">
              {item.labels.map((label) => {
                const colors = labelColor(label.color);
                return (
                  <span
                    key={label.id}
                    className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
                    style={{ backgroundColor: colors.bg, color: colors.text }}
                  >
                    {label.name}
                  </span>
                );
              })}
            </div>
          </div>
        )}

        {/* Body */}
        <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
          {item.body ? (
            <div className="max-h-64 overflow-y-auto rounded-md bg-gray-50 p-4 text-sm text-gray-800 dark:bg-gray-800 dark:text-gray-200" style={{ whiteSpace: "pre-wrap" }}>
              {item.body}
            </div>
          ) : (
            <p className="text-sm italic text-gray-400 dark:text-gray-500">No description provided.</p>
          )}
        </div>
      </div>
    </div>
  );
}
