"use client";

import { useState } from "react";
import { useRepos } from "@/hooks/use-repos";
import { addTrackedRepo, removeTrackedRepo, syncRepo } from "@/lib/api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:9423";
const MAX_REPOS = 5;

export default function SettingsPage() {
  const { repos, mutate } = useRepos();
  const [repoInput, setRepoInput] = useState("");
  const [adding, setAdding] = useState(false);
  const [addError, setAddError] = useState("");
  const [removingRepo, setRemovingRepo] = useState<string | null>(null);
  const [syncingRepo, setSyncingRepo] = useState<string | null>(null);

  async function handleAddRepo(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = repoInput.trim();
    if (!trimmed) return;

    const parts = trimmed.split("/");
    if (parts.length !== 2 || !parts[0] || !parts[1]) {
      setAddError("Invalid format. Use owner/repo");
      return;
    }

    setAdding(true);
    setAddError("");
    try {
      await addTrackedRepo(trimmed);
      setRepoInput("");
      await mutate();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Failed to add repository";
      setAddError(message);
    } finally {
      setAdding(false);
    }
  }

  async function handleRemoveRepo(owner: string, name: string) {
    const fullName = `${owner}/${name}`;
    setRemovingRepo(fullName);
    try {
      await removeTrackedRepo(owner, name);
      await mutate();
    } catch (err) {
      console.error("Failed to remove repo:", err);
    } finally {
      setRemovingRepo(null);
    }
  }

  async function handleSyncRepo(owner: string, name: string) {
    const fullName = `${owner}/${name}`;
    setSyncingRepo(fullName);
    try {
      await syncRepo(owner, name);
      await mutate();
    } catch (err) {
      console.error("Sync failed:", err);
    } finally {
      setSyncingRepo(null);
    }
  }

  const atLimit = repos.length >= MAX_REPOS;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Settings
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Configure your greport dashboard
        </p>
      </div>

      {/* Repository Management */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Tracked Repositories
        </h3>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Manage repositories tracked by greport (up to {MAX_REPOS}).
        </p>

        {/* Repo list */}
        <div className="mt-4 space-y-3">
          {repos.length === 0 ? (
            <p className="text-sm text-gray-400 dark:text-gray-500">
              No repositories tracked yet. Add one below.
            </p>
          ) : (
            repos.map((r) => (
              <div
                key={r.fullName}
                className="flex items-center justify-between rounded-md border border-gray-200 bg-gray-50 px-4 py-3 dark:border-gray-700 dark:bg-gray-800"
              >
                <div className="flex items-center gap-3">
                  <RepoIcon className="h-4 w-4 text-gray-400" />
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {r.fullName}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleSyncRepo(r.owner, r.name)}
                    disabled={syncingRepo === r.fullName}
                    className="rounded-md px-2.5 py-1 text-xs font-medium text-blue-600 hover:bg-blue-50 disabled:opacity-50 dark:text-blue-400 dark:hover:bg-blue-900/20"
                  >
                    {syncingRepo === r.fullName ? "Syncing..." : "Sync"}
                  </button>
                  <button
                    onClick={() => handleRemoveRepo(r.owner, r.name)}
                    disabled={removingRepo === r.fullName}
                    className="rounded-md px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 disabled:opacity-50 dark:text-red-400 dark:hover:bg-red-900/20"
                  >
                    {removingRepo === r.fullName ? "Removing..." : "Remove"}
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Add repo form */}
        <form onSubmit={handleAddRepo} className="mt-4 flex items-center gap-3">
          <input
            type="text"
            value={repoInput}
            onChange={(e) => { setRepoInput(e.target.value); setAddError(""); }}
            placeholder="owner/repo"
            disabled={atLimit}
            className="w-64 rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:bg-gray-100 disabled:text-gray-400 dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:disabled:bg-gray-900 dark:disabled:text-gray-600"
          />
          <button
            type="submit"
            disabled={atLimit || adding || !repoInput.trim()}
            className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {adding ? "Adding..." : "Add Repository"}
          </button>
        </form>
        {atLimit && (
          <p className="mt-2 text-sm text-amber-600 dark:text-amber-400">
            Maximum {MAX_REPOS} repositories reached. Remove a repository to add a new one.
          </p>
        )}
        {addError && (
          <p className="mt-2 text-sm text-red-600 dark:text-red-400">
            {addError}
          </p>
        )}
      </div>

      {/* API Configuration */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          API Configuration
        </h3>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          The API URL is configured via environment variable NEXT_PUBLIC_API_URL.
        </p>
        <div className="mt-4">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            API Base URL
          </label>
          <input
            type="text"
            value={API_BASE}
            readOnly
            className="mt-1 w-64 rounded-md border border-gray-300 bg-gray-50 px-3 py-2 text-sm text-gray-500 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-400"
          />
          <p className="mt-1 text-xs text-gray-400 dark:text-gray-500">
            Set NEXT_PUBLIC_API_URL in your .env.local file to change this.
          </p>
        </div>
      </div>

      {/* SLA Configuration */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          SLA Defaults
        </h3>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Default SLA thresholds are configured on the API server via environment
          variables.
        </p>
        <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Response SLA (hours)
            </label>
            <p className="mt-1 text-sm text-gray-500">
              Configure via SLA_RESPONSE_HOURS on the API server (default: 24h)
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Resolution SLA (hours)
            </label>
            <p className="mt-1 text-sm text-gray-500">
              Configure via SLA_RESOLUTION_HOURS on the API server (default: 168h)
            </p>
          </div>
        </div>
      </div>

      {/* About */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          About
        </h3>
        <dl className="mt-4 space-y-2 text-sm">
          <div className="flex gap-2">
            <dt className="font-medium text-gray-500 dark:text-gray-400">Version:</dt>
            <dd className="text-gray-900 dark:text-white">0.4.0</dd>
          </div>
          <div className="flex gap-2">
            <dt className="font-medium text-gray-500 dark:text-gray-400">Source:</dt>
            <dd>
              <a
                href="https://github.com/prabasiva/greport"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300"
              >
                github.com/prabasiva/greport
              </a>
            </dd>
          </div>
        </dl>
      </div>
    </div>
  );
}

function RepoIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
    </svg>
  );
}
