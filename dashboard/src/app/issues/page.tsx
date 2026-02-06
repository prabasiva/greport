"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useIssues, useIssueMetrics } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { Pagination } from "@/components/shared/pagination";
import { FilterPanel, ExportButton } from "@/components/shared/filter-panel";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { MetricCard } from "@/components/shared/metric-card";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime, formatDate, labelColor, exportToCsv, exportToJson } from "@/lib/utils";
import type { Issue } from "@/types/api";

export default function IssuesPage() {
  const { owner, repo } = useRepo();

  if (!owner || !repo) return <NoRepoSelected />;

  return <IssuesContent owner={owner} repo={repo} />;
}

function IssuesContent({ owner, repo }: { owner: string; repo: string }) {
  const [state, setState] = useState("open");
  const [page, setPage] = useState(1);
  const perPage = 30;

  const { data, error, isLoading } = useIssues(owner, repo, {
    state,
    page,
    per_page: perPage,
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
        ]}
        onClear={() => { setState("open"); setPage(1); }}
      />

      {/* Issue Table */}
      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={issues}
          keyExtractor={(i) => i.id}
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
    </div>
  );
}
