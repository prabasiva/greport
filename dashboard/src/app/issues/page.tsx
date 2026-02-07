"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useRepos } from "@/hooks/use-repos";
import { useIssues, useIssueMetrics, useAggregateIssueMetrics, useAggregateIssues } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { Pagination } from "@/components/shared/pagination";
import { FilterPanel, ExportButton } from "@/components/shared/filter-panel";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { MetricCard } from "@/components/shared/metric-card";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime, formatDate, labelColor, exportToCsv, exportToJson, formatHours } from "@/lib/utils";
import { DetailPopover } from "@/components/shared/detail-popover";
import type { Issue, AggregateIssueItem } from "@/types/api";

export default function IssuesPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateIssuesView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;

  return <IssuesContent owner={owner} repo={repo} />;
}

function AggregateIssuesView() {
  const [state, setState] = useState("all");
  const [days, setDays] = useState("all");
  const [page, setPage] = useState(1);
  const [selectedIssue, setSelectedIssue] = useState<AggregateIssueItem | null>(null);
  const perPage = 30;

  const filterParams: { state?: string; days?: number } = {};
  if (state !== "all") filterParams.state = state;
  if (days !== "all") filterParams.days = Number(days);

  const { data: metricsData } = useAggregateIssueMetrics(
    Object.keys(filterParams).length > 0 ? filterParams : undefined,
  );
  const { data: listData, error, isLoading } = useAggregateIssues({
    ...filterParams,
    page,
    per_page: perPage,
  });

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const metrics = metricsData?.data;
  const issues = listData?.data || [];
  const meta = listData?.meta;

  const labelData = metrics
    ? Object.entries(metrics.by_label)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10)
        .map(([name, value]) => ({ name, value }))
    : [];

  const assigneeData = metrics
    ? Object.entries(metrics.by_assignee)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10)
        .map(([name, value]) => ({ name, value }))
    : [];

  const columns: Column<AggregateIssueItem>[] = [
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
          <p className="truncate font-medium text-gray-900 dark:text-white">
            {item.title}
          </p>
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
      render: (item) => (
        <span
          className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
            item.state === "open"
              ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
              : "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
          }`}
        >
          {item.state}
        </span>
      ),
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
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Issues</h2>
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
        ]}
        onClear={() => { setState("all"); setDays("all"); setPage(1); }}
      />

      {metrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard title="Total" value={metrics.totals.total} />
          <MetricCard title="Open" value={metrics.totals.open} />
          <MetricCard title="Closed" value={metrics.totals.closed} />
          <MetricCard title="Stale" value={metrics.totals.stale_count} />
        </div>
      )}

      {/* Issue list */}
      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={issues}
          keyExtractor={(i) => `${i.repository}-${i.id}`}
          onRowClick={(item) => setSelectedIssue(item)}
          emptyMessage="No issues found matching the filters"
        />
        {meta && <Pagination meta={meta} onPageChange={setPage} />}
      </div>

      {metrics && (
        <>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Issues by Repository</h3>
              <BarChartComponent
                data={metrics.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.total,
                }))}
              />
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Open Issues by Repository</h3>
              <BarChartComponent
                data={metrics.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.open,
                }))}
                color="#ef4444"
              />
            </div>
          </div>

          {/* Distribution charts */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {metrics.age_distribution.length > 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
                <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Age Distribution</h3>
                <BarChartComponent
                  data={metrics.age_distribution.map((b) => ({
                    name: b.label,
                    value: b.count,
                  }))}
                  layout="horizontal"
                  color="#8b5cf6"
                />
              </div>
            )}
            {labelData.length > 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
                <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Issues by Label</h3>
                <BarChartComponent data={labelData} color="#f59e0b" />
              </div>
            )}
            {assigneeData.length > 0 && (
              <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
                <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">Issues by Assignee</h3>
                <BarChartComponent data={assigneeData} color="#06b6d4" />
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
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Closed</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Avg Close</th>
                    <th className="px-4 py-2 text-right font-medium text-gray-500">Stale</th>
                  </tr>
                </thead>
                <tbody>
                  {metrics.by_repository.map((r) => (
                    <tr key={r.repository} className="border-b border-gray-100 dark:border-gray-800">
                      <td className="px-4 py-2 font-medium text-gray-900 dark:text-white">{r.repository}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.total}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.open}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.closed}</td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">
                        {r.avg_time_to_close_hours ? formatHours(r.avg_time_to_close_hours) : "N/A"}
                      </td>
                      <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.stale_count}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}

      {selectedIssue && (() => {
        const parts = selectedIssue.repository.split("/");
        return (
          <DetailPopover
            item={selectedIssue}
            owner={parts[0]}
            repo={parts[1]}
            onClose={() => setSelectedIssue(null)}
          />
        );
      })()}
    </div>
  );
}

function IssuesContent({ owner, repo }: { owner: string; repo: string }) {
  const [state, setState] = useState("open");
  const [days, setDays] = useState("all");
  const [page, setPage] = useState(1);
  const [selectedIssue, setSelectedIssue] = useState<Issue | null>(null);
  const perPage = 30;

  const { data, error, isLoading } = useIssues(owner, repo, {
    state,
    page,
    per_page: perPage,
    days: days !== "all" ? Number(days) : undefined,
  });
  const { data: metricsData } = useIssueMetrics(owner, repo);

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const issues = data?.data || [];
  const meta = data?.meta;
  const metrics = metricsData?.data;

  const columns: Column<Issue>[] = [
    {
      key: "number",
      header: "#",
      sortable: true,
      className: "w-16",
      render: (issue) => (
        <span className="font-mono text-xs text-gray-500">#{issue.number}</span>
      ),
    },
    {
      key: "title",
      header: "Title",
      render: (issue) => (
        <div className="max-w-md">
          <p className="truncate font-medium text-gray-900 dark:text-white">
            {issue.title}
          </p>
          <div className="mt-1 flex flex-wrap gap-1">
            {issue.labels.map((label) => {
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
      ),
    },
    {
      key: "state",
      header: "State",
      sortable: true,
      render: (issue) => (
        <span
          className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
            issue.state === "open"
              ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
              : "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
          }`}
        >
          {issue.state}
        </span>
      ),
    },
    {
      key: "author",
      header: "Author",
      render: (issue) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {issue.author.login}
        </span>
      ),
    },
    {
      key: "comments_count",
      header: "Comments",
      sortable: true,
      className: "w-24",
      render: (issue) => (
        <span className="text-sm text-gray-500">{issue.comments_count}</span>
      ),
    },
    {
      key: "created_at",
      header: "Created",
      sortable: true,
      render: (issue) => (
        <span className="text-sm text-gray-500" title={formatDate(issue.created_at)}>
          {formatRelativeTime(issue.created_at)}
        </span>
      ),
    },
  ];

  function handleExportCsv() {
    const rows = issues.map((i) => ({
      number: i.number,
      title: i.title,
      state: i.state,
      author: i.author.login,
      labels: i.labels.map((l) => l.name).join(";"),
      assignees: i.assignees.map((a) => a.login).join(";"),
      comments: i.comments_count,
      created_at: i.created_at,
      updated_at: i.updated_at,
      closed_at: i.closed_at || "",
    }));
    exportToCsv(rows, `${owner}-${repo}-issues`);
  }

  function handleExportJson() {
    exportToJson(issues, `${owner}-${repo}-issues`);
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Issues
          </h2>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            {owner}/{repo}
          </p>
        </div>
        <ExportButton onExportCsv={handleExportCsv} onExportJson={handleExportJson} />
      </div>

      {/* Quick metrics */}
      {metrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard title="Total" value={metrics.total} />
          <MetricCard title="Open" value={metrics.open} />
          <MetricCard title="Closed" value={metrics.closed} />
          <MetricCard title="Stale" value={metrics.stale_count} />
        </div>
      )}

      {/* Filters */}
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

      {/* Issue Table */}
      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={issues}
          keyExtractor={(i) => i.id}
          onRowClick={(item) => setSelectedIssue(item)}
          emptyMessage="No issues found matching the filters"
        />
        {meta && <Pagination meta={meta} onPageChange={setPage} />}
      </div>

      {/* Age Distribution Chart */}
      {metrics && metrics.age_distribution.buckets.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Age Distribution
          </h3>
          <BarChartComponent
            data={metrics.age_distribution.buckets.map((b) => ({
              name: b.label,
              value: b.count,
            }))}
            layout="horizontal"
            color="#8b5cf6"
          />
        </div>
      )}

      {selectedIssue && (
        <DetailPopover
          item={selectedIssue}
          owner={owner}
          repo={repo}
          onClose={() => setSelectedIssue(null)}
        />
      )}
    </div>
  );
}
