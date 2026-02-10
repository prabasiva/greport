"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useContributors, useAggregateContributors } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { FilterPanel } from "@/components/shared/filter-panel";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import type { ContributorStats, AggregateContributorStats } from "@/types/api";

export default function ContributorsPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateContributorsView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;
  return <ContributorsContent owner={owner} repo={repo} />;
}

function AggregateContributorsView() {
  const [sortBy, setSortBy] = useState("prs");
  const [repoFilter, setRepoFilter] = useState("all");
  const [authorFilter, setAuthorFilter] = useState("all");

  const { data, error, isLoading } = useAggregateContributors();

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const contributors = data?.data || [];

  const allRepos = [...new Set(contributors.flatMap(c => c.repositories))].sort();
  const allAuthors = [...new Set(contributors.map(c => c.login))].sort();

  const filtered = contributors.filter(c => {
    if (repoFilter !== "all" && !c.repositories.includes(repoFilter)) return false;
    if (authorFilter !== "all" && c.login !== authorFilter) return false;
    return true;
  });

  const sorted = [...filtered].sort((a, b) => {
    if (sortBy === "prs") return b.total_prs_merged - a.total_prs_merged;
    return b.total_issues_created - a.total_issues_created;
  });

  const columns: Column<AggregateContributorStats>[] = [
    {
      key: "rank",
      header: "#",
      className: "w-12",
      render: (c) => {
        const idx = sorted.indexOf(c);
        return <span className="text-sm font-medium text-gray-500">{idx + 1}</span>;
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
      key: "repositories",
      header: "Repositories",
      render: (c) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {c.repositories.length}
        </span>
      ),
    },
    {
      key: "issues_created",
      header: "Issues Created",
      sortable: true,
      render: (c) => <span className="text-sm">{c.total_issues_created}</span>,
    },
    {
      key: "prs_created",
      header: "PRs Created",
      sortable: true,
      render: (c) => <span className="text-sm">{c.total_prs_created}</span>,
    },
    {
      key: "prs_merged",
      header: "PRs Merged",
      sortable: true,
      render: (c) => <span className="text-sm">{c.total_prs_merged}</span>,
    },
    {
      key: "total",
      header: "Total Activity",
      sortable: true,
      render: (c) => (
        <span className="text-sm font-medium">
          {c.total_issues_created + c.total_prs_created}
        </span>
      ),
    },
  ];

  const chartData = sorted.slice(0, 10).map((c) => ({
    name: c.login,
    value: sortBy === "prs" ? c.total_prs_merged : c.total_issues_created,
  }));

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Contributors
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          All Repositories
        </p>
      </div>

      <FilterPanel
        filters={[
          {
            key: "sortBy",
            label: "Sort by",
            value: sortBy,
            onChange: setSortBy,
            options: [
              { label: "PRs Merged", value: "prs" },
              { label: "Issues Created", value: "issues" },
            ],
          },
          {
            key: "repo",
            label: "Repository",
            value: repoFilter,
            onChange: setRepoFilter,
            options: [
              { label: "All", value: "all" },
              ...allRepos.map(r => ({ label: r.split("/").pop() || r, value: r })),
            ],
          },
          {
            key: "author",
            label: "Contributor",
            value: authorFilter,
            onChange: setAuthorFilter,
            options: [
              { label: "All", value: "all" },
              ...allAuthors.map(a => ({ label: a, value: a })),
            ],
          },
        ]}
        onClear={() => { setSortBy("prs"); setRepoFilter("all"); setAuthorFilter("all"); }}
      />

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
          data={sorted}
          keyExtractor={(c) => c.login}
          emptyMessage="No contributor data available"
        />
      </div>
    </div>
  );
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
