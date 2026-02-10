"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useRepos } from "@/hooks/use-repos";
import { usePulls, usePullMetrics, useAggregatePullMetrics, useAggregatePulls } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { Pagination } from "@/components/shared/pagination";
import { FilterPanel } from "@/components/shared/filter-panel";
import { formatDate } from "@/lib/utils";
import { MetricCard } from "@/components/shared/metric-card";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { PieChartComponent } from "@/components/charts/pie-chart-component";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime, formatHours } from "@/lib/utils";
import { DetailPopover } from "@/components/shared/detail-popover";
import type { PullRequest, AggregatePullItem } from "@/types/api";

export default function PullsPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregatePullsView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;
  return <PullsContent owner={owner} repo={repo} />;
}

function AggregatePullsView() {
  const [state, setState] = useState("all");
  const [days, setDays] = useState("all");
  const [repoFilter, setRepoFilter] = useState("all");
  const [authorFilter, setAuthorFilter] = useState("all");
  const [page, setPage] = useState(1);
  const [selectedPull, setSelectedPull] = useState<AggregatePullItem | null>(null);
  const perPage = 30;

  const filterParams: { state?: string; days?: number } = {};
  if (state !== "all") filterParams.state = state;
  if (days !== "all") filterParams.days = Number(days);

  const { data: metricsData } = useAggregatePullMetrics(
    Object.keys(filterParams).length > 0 ? filterParams : undefined,
  );
  const { data: listData, error, isLoading } = useAggregatePulls({
    ...filterParams,
    page,
    per_page: perPage,
  });

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const metrics = metricsData?.data;
  const pulls = listData?.data || [];
  const meta = listData?.meta;

  const repos = [...new Set(pulls.map(p => p.repository))].sort();
  const authors = [...new Set(pulls.map(p => p.author.login))].sort();

  const filtered = pulls.filter(p => {
    if (repoFilter !== "all" && p.repository !== repoFilter) return false;
    if (authorFilter !== "all" && p.author.login !== authorFilter) return false;
    return true;
  });

  const sizeData = metrics
    ? Object.entries(metrics.by_size).map(([name, value]) => ({ name, value }))
    : [];

  const authorData = metrics
    ? Object.entries(metrics.by_author)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10)
        .map(([name, value]) => ({ name, value }))
    : [];

  const columns: Column<AggregatePullItem>[] = [
    {
      key: "number",
      header: "#",
      sortable: true,
      className: "w-16",
      render: (item) => (
        <span className="font-mono text-xs text-gray-500">#{item.number}</span>
      ),
    },
    {
      key: "title",
      header: "Title",
      render: (item) => (
        <div className="max-w-md">
          <div className="flex items-center gap-2">
            <p className="truncate font-medium text-gray-900 dark:text-white">
              {item.title}
            </p>
            {item.draft && (
              <span className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500 dark:bg-gray-800">
                Draft
              </span>
            )}
          </div>
          <p className="mt-0.5 text-xs text-gray-500">
            {item.head_ref} &rarr; {item.base_ref}
          </p>
        </div>
      ),
    },
    {
      key: "repository",
      header: "Repository",
      render: (item) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {item.repository.split("/").pop() || item.repository}
        </span>
      ),
    },
    {
      key: "state",
      header: "State",
      sortable: true,
      render: (item) => {
        const merged = item.merged;
        const label = merged ? "merged" : item.state;
        const cls = merged
          ? "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
          : item.state === "open"
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
      render: (item) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {item.author.login}
        </span>
      ),
    },
    {
      key: "changes",
      header: "Changes",
      render: (item) => (
        <span className="text-sm">
          <span className="text-green-600">+{item.additions}</span>
          {" / "}
          <span className="text-red-600">-{item.deletions}</span>
        </span>
      ),
    },
    {
      key: "created_at",
      header: "Created",
      sortable: true,
      render: (item) => (
        <span className="text-sm text-gray-500" title={formatDate(item.created_at)}>
          {formatRelativeTime(item.created_at)}
        </span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Pull Requests</h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">All Repositories</p>
      </div>

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
          {
            key: "days",
            label: "Time Frame",
            value: days,
            onChange: (v) => { setDays(v); setPage(1); },
            options: [
              { label: "30 days", value: "30" },
              { label: "60 days", value: "60" },
              { label: "90 days", value: "90" },
              { label: "120 days", value: "120" },
              { label: "240 days", value: "240" },
              { label: "All time", value: "all" },
            ],
          },
          {
            key: "repo",
            label: "Repository",
            value: repoFilter,
            onChange: (v) => { setRepoFilter(v); setPage(1); },
            options: [
              { label: "All", value: "all" },
              ...repos.map(r => ({ label: r.split("/").pop() || r, value: r })),
            ],
          },
          {
            key: "author",
            label: "Author",
            value: authorFilter,
            onChange: (v) => { setAuthorFilter(v); setPage(1); },
            options: [
              { label: "All", value: "all" },
              ...authors.map(a => ({ label: a, value: a })),
            ],
          },
        ]}
        onClear={() => { setState("all"); setDays("all"); setRepoFilter("all"); setAuthorFilter("all"); setPage(1); }}
      />

      {metrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard title="Total PRs" value={metrics.totals.total} />
          <MetricCard title="Open" value={metrics.totals.open} />
          <MetricCard title="Merged" value={metrics.totals.merged} />
          <MetricCard
            title="Avg Time to Merge"
            value={metrics.totals.avg_time_to_merge_hours ? formatHours(metrics.totals.avg_time_to_merge_hours) : "N/A"}
          />
        </div>
      )}

      {/* PR list */}
      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={filtered}
          keyExtractor={(pr) => `${pr.repository}-${pr.id}`}
          onRowClick={(item) => setSelectedPull(item)}
          emptyMessage="No pull requests found matching the filters"
        />
        {meta && <Pagination meta={meta} onPageChange={setPage} />}
      </div>

      {metrics && (
        <>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">PRs by Repository</h3>
              <BarChartComponent
                data={metrics.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.total,
                }))}
              />
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Merged PRs by Repository</h3>
              <BarChartComponent
                data={metrics.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.merged,
                }))}
                color="#22c55e"
              />
            </div>
          </div>

          {/* Distribution charts */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {sizeData.length > 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
                <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">PR Size Distribution</h3>
                <PieChartComponent data={sizeData} />
              </div>
            )}
            {authorData.length > 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
                <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">PRs by Author</h3>
                <BarChartComponent data={authorData} color="#f59e0b" />
              </div>
            )}
          </div>

          {/* Per-repo breakdown table */}
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Per-Repository Breakdown</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-200 dark:border-gray-700">
                    <th className="px-4 py-2 text-left font-medium text-gray-500">Repository</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Total</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Open</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Merged</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Avg Merge</th>
                  </tr>
                </thead>
                <tbody>
                  {metrics.by_repository.map((r) => (
                    <tr key={r.repository} className="border-b border-gray-100 dark:border-gray-800">
                      <td className="px-4 py-2 font-medium text-gray-900 dark:text-white">{r.repository}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.total}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.open}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.merged}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">
                        {r.avg_time_to_merge_hours ? formatHours(r.avg_time_to_merge_hours) : "N/A"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}

      {selectedPull && (() => {
        const parts = selectedPull.repository.split("/");
        return (
          <DetailPopover
            item={selectedPull}
            owner={parts[0]}
            repo={parts[1]}
            onClose={() => setSelectedPull(null)}
          />
        );
      })()}
    </div>
  );
}

function PullsContent({ owner, repo }: { owner: string; repo: string }) {
  const [state, setState] = useState("open");
  const [days, setDays] = useState("all");
  const [page, setPage] = useState(1);
  const [selectedPull, setSelectedPull] = useState<PullRequest | null>(null);

  const { data, error, isLoading } = usePulls(owner, repo, {
    state,
    page,
    per_page: 30,
    days: days !== "all" ? Number(days) : undefined,
  });
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
          {
            key: "days",
            label: "Time Frame",
            value: days,
            onChange: (v) => { setDays(v); setPage(1); },
            options: [
              { label: "30 days", value: "30" },
              { label: "60 days", value: "60" },
              { label: "90 days", value: "90" },
              { label: "120 days", value: "120" },
              { label: "240 days", value: "240" },
              { label: "All time", value: "all" },
            ],
          },
        ]}
        onClear={() => { setState("open"); setDays("all"); setPage(1); }}
      />

      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={pulls}
          keyExtractor={(pr) => pr.id}
          onRowClick={(item) => setSelectedPull(item)}
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

      {selectedPull && (
        <DetailPopover
          item={selectedPull}
          owner={owner}
          repo={repo}
          onClose={() => setSelectedPull(null)}
        />
      )}
    </div>
  );
}
