"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useContributors } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import type { ContributorStats } from "@/types/api";

export default function ContributorsPage() {
  const { owner, repo } = useRepo();
  if (!owner || !repo) return <NoRepoSelected />;
  return <ContributorsContent owner={owner} repo={repo} />;
}

function ContributorsContent({ owner, repo }: { owner: string; repo: string }) {
  const [sortBy, setSortBy] = useState("prs");
  const [limit, setLimit] = useState(20);

  const { data, error, isLoading } = useContributors(owner, repo, { sort_by: sortBy, limit });

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const contributors = data?.data || [];

  const columns: Column<ContributorStats>[] = [
    {
      key: "rank",
      header: "#",
      className: "w-12",
      render: (_, ) => {
        const idx = contributors.indexOf(_);
        return (
          <span className="text-sm font-medium text-gray-500">{idx + 1}</span>
        );
      },
    },
    {
      key: "login",
      header: "Contributor",
      render: (c) => (
        <span className="font-medium text-gray-900 dark:text-white">
          {c.login}
        </span>
      ),
    },
    {
      key: "issues_created",
      header: "Issues Created",
      sortable: true,
      render: (c) => <span className="text-sm">{c.issues_created}</span>,
    },
    {
      key: "prs_created",
      header: "PRs Created",
      sortable: true,
      render: (c) => <span className="text-sm">{c.prs_created}</span>,
    },
    {
      key: "prs_merged",
      header: "PRs Merged",
      sortable: true,
      render: (c) => <span className="text-sm">{c.prs_merged}</span>,
    },
    {
      key: "total",
      header: "Total Activity",
      sortable: true,
      render: (c) => (
        <span className="text-sm font-medium">
          {c.issues_created + c.prs_created}
        </span>
      ),
    },
  ];

  const chartData = contributors.slice(0, 10).map((c) => ({
    name: c.login,
    value: sortBy === "prs" ? c.prs_merged : c.issues_created,
  }));

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Contributors
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {owner}/{repo}
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-4">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Sort by
          </label>
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value)}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value="prs">PRs Merged</option>
            <option value="issues">Issues Created</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Limit
          </label>
          <select
            value={limit}
            onChange={(e) => setLimit(Number(e.target.value))}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value={10}>10</option>
            <option value={20}>20</option>
            <option value={50}>50</option>
          </select>
        </div>
      </div>

      {chartData.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Top Contributors ({sortBy === "prs" ? "PRs Merged" : "Issues Created"})
          </h3>
          <BarChartComponent data={chartData} color="#06b6d4" />
        </div>
      )}

      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={contributors}
          keyExtractor={(c) => c.login}
          emptyMessage="No contributor data available"
        />
      </div>
    </div>
  );
}
