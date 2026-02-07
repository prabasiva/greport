"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";

export default function SettingsPage() {
  const { owner, repo, setRepo } = useRepo();
  const [repoInput, setRepoInput] = useState(`${owner}/${repo}`);
  const [apiUrl, setApiUrl] = useState(
    process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000",
  );
  const [saved, setSaved] = useState(false);

  function handleSaveRepo(e: React.FormEvent) {
    e.preventDefault();
    const parts = repoInput.split("/");
    if (parts.length === 2 && parts[0] && parts[1]) {
      setRepo(parts[0], parts[1]);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    }
  }

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

      {/* Repository Settings */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Repository
        </h3>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          Set the default repository to analyze.
        </p>
        <form onSubmit={handleSaveRepo} className="mt-4 flex items-center gap-3">
          <input
            type="text"
            value={repoInput}
            onChange={(e) => setRepoInput(e.target.value)}
            placeholder="owner/repo"
            className="w-64 rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          />
          <button
            type="submit"
            className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500"
          >
            Save
          </button>
          {saved && (
            <span className="text-sm text-green-600">Saved</span>
          )}
        </form>
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
            value={apiUrl}
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
            <dd className="text-gray-900 dark:text-white">0.3.0</dd>
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
