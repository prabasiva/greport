"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { usePulls, usePullMetrics } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { Pagination } from "@/components/shared/pagination";
import { FilterPanel } from "@/components/shared/filter-panel";
import { MetricCard } from "@/components/shared/metric-card";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { PieChartComponent } from "@/components/charts/pie-chart-component";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime, formatHours } from "@/lib/utils";
import type { PullRequest } from "@/types/api";

export default function PullsPage() {
  const { owner, repo } = useRepo();
  if (!owner || !repo) return <NoRepoSelected />;
  return <PullsContent owner={owner} repo={repo} />;
}

function PullsContent({ owner, repo }: { owner: string; repo: string }) {
  const [state, setState] = useState("open");
  const [page, setPage] = useState(1);

  const { data, error, isLoading } = usePulls(owner, repo, { state, page, per_page: 30 });
  const { data: metricsData } = usePullMetrics(owner, repo);

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const pulls = data?.data || [];
  const meta = data?.meta;
  const metrics = metricsData?.data;

  const columns: Column<PullRequest>[] = [
    {
      key: "number",
      header: "#",
      sortable: true,
      className: "w-16",
      render: (pr) => (
        <span className="font-mono text-xs text-gray-500">#{pr.number}</span>
      ),
    },
    {
      key: "title",
      header: "Title",
      render: (pr) => (
        <div className="max-w-md">
          <div className="flex items-center gap-2">
            <p className="truncate font-medium text-gray-900 dark:text-white">
              {pr.title}
            </p>
            {pr.draft && (
              <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500 dark:bg-gray-800">
                Draft
              </span>
            )}
          </div>
          <p className="mt-0.5 text-xs text-gray-500">
            {pr.head_ref} &rarr; {pr.base_ref}
          </p>
        </div>
      ),
    },
    {
      key: "state",
      header: "State",
      sortable: true,
      render: (pr) => {
        const merged = pr.merged;
        const label = merged ? "merged" : pr.state;
        const cls = merged
          ? "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
          : pr.state === "open"
            ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
            : "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300";
        return (
          <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${cls}`}>
            {label}
          </span>
        );
      },
    },
    {
      key: "author",
      header: "Author",
      render: (pr) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {pr.author.login}
        </span>
      ),
    },
    {
      key: "changes",
      header: "Changes",
      render: (pr) => (
        <span className="text-sm">
          <span className="text-green-600">+{pr.additions}</span>
          {" / "}
          <span className="text-red-600">-{pr.deletions}</span>
        </span>
      ),
    },
    {
      key: "created_at",
      header: "Created",
      sortable: true,
      render: (pr) => (
        <span className="text-sm text-gray-500">
          {formatRelativeTime(pr.created_at)}
        </span>
      ),
    },
  ];

  const sizeData = metrics
    ? Object.entries(metrics.by_size)
        .map(([name, value]) => ({ name, value }))
    : [];

  const authorData = metrics
    ? Object.entries(metrics.by_author)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10)
        .map(([name, value]) => ({ name, value }))
    : [];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Pull Requests
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {owner}/{repo}
        </p>
      </div>

      {metrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard title="Total PRs" value={metrics.total} />
          <MetricCard title="Open" value={metrics.open} />
          <MetricCard title="Merged" value={metrics.merged} />
          <MetricCard
            title="Avg Time to Merge"
            value={metrics.avg_time_to_merge_hours ? formatHours(metrics.avg_time_to_merge_hours) : "N/A"}
            subtitle={metrics.median_time_to_merge_hours ? `Median: ${formatHours(metrics.median_time_to_merge_hours)}` : undefined}
          />
        </div>
      )}

      <FilterPanel
        filters={[
          {
            key: "state",
            label: "State",
            value: state,
            onChange: (v) => { setState(v); setPage(1); },
            options: [
              { label: "Open", value: "open" },
              { label: "Closed", value: "closed" },
              { label: "All", value: "all" },
            ],
          },
        ]}
        onClear={() => { setState("open"); setPage(1); }}
      />

      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={pulls}
          keyExtractor={(pr) => pr.id}
          emptyMessage="No pull requests found"
        />
        {meta && <Pagination meta={meta} onPageChange={setPage} />}
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {sizeData.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              PR Size Distribution
            </h3>
            <PieChartComponent data={sizeData} />
          </div>
        )}
        {authorData.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              PRs by Author
            </h3>
            <BarChartComponent data={authorData} color="#f59e0b" />
          </div>
        )}
      </div>
    </div>
  );
}
