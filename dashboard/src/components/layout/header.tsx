"use client";

import { useState } from "react";
import { useSWRConfig } from "swr";
import { useRepo } from "@/hooks/use-repo";
import { useRepos } from "@/hooks/use-repos";
import { syncRepo, batchSync } from "@/lib/api";

export function Header() {
  const { owner, repo, mode, setRepo, setMode } = useRepo();
  const { repos } = useRepos();
  const [syncing, setSyncing] = useState(false);
  const { mutate } = useSWRConfig();

  function handleRepoChange(value: string) {
    if (value === "__aggregate__") {
      setMode("aggregate");
    } else {
      const parts = value.split("/");
      if (parts.length === 2 && parts[0] && parts[1]) {
        setRepo(parts[0], parts[1]);
        setMode("single");
      }
    }
  }

  async function handleSync() {
    if (syncing) return;
    setSyncing(true);
    try {
      if (mode === "aggregate") {
        await batchSync();
      } else if (owner && repo) {
        await syncRepo(owner, repo);
      }
      await mutate(() => true, undefined, { revalidate: true });
    } catch (err) {
      console.error("Sync failed:", err);
    } finally {
      setSyncing(false);
    }
  }

  const currentValue = mode === "aggregate" ? "__aggregate__" : (owner && repo ? `${owner}/${repo}` : "");

  return (
    <header className="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-x-4 border-b border-gray-200 bg-white px-4 shadow-sm dark:border-gray-800 dark:bg-gray-950 sm:gap-x-6 sm:px-6 lg:px-8">
      {/* Mobile menu button */}
      <MobileMenuButton />

      <div className="flex flex-1 gap-x-4 self-stretch lg:gap-x-6">
        <div className="flex flex-1 items-center gap-x-4">
          <div className="lg:hidden text-lg font-bold text-gray-900 dark:text-white">
            greport
          </div>
          <div className="hidden lg:block" />
        </div>

        <div className="flex items-center gap-x-4 lg:gap-x-6">
          <div className="flex items-center gap-2">
            <RepoIcon className="h-4 w-4 text-gray-500" />
            <select
              value={currentValue}
              onChange={(e) => handleRepoChange(e.target.value)}
              className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-900 shadow-sm hover:bg-gray-50 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:hover:bg-gray-700"
            >
              <option value="__aggregate__">All Repositories</option>
              {repos.map((r) => (
                <option key={r.fullName} value={r.fullName}>
                  {r.fullName}
                </option>
              ))}
              {owner && repo && !repos.find((r) => r.fullName === `${owner}/${repo}`) && (
                <option value={`${owner}/${repo}`}>{owner}/{repo}</option>
              )}
            </select>
          </div>

          <button
            onClick={handleSync}
            disabled={syncing}
            title={mode === "aggregate" ? "Sync all repositories" : "Sync data from GitHub"}
            className="flex items-center gap-1.5 rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
          >
            <RefreshIcon className={`h-4 w-4 ${syncing ? "animate-spin" : ""}`} />
            {syncing ? "Syncing..." : "Refresh"}
          </button>
        </div>
      </div>
    </header>
  );
}

function MobileMenuButton() {
  return (
    <button
      type="button"
      className="-m-2.5 p-2.5 text-gray-700 lg:hidden dark:text-gray-300"
      aria-label="Open sidebar"
    >
      <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
      </svg>
    </button>
  );
}

function RepoIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
    </svg>
  );
}

function RefreshIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182M20.984 4.356v4.992" />
    </svg>
  );
}
