"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useRepo } from "@/hooks/use-repo";
import { useProjects, useAggregateProjects } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { FilterPanel } from "@/components/shared/filter-panel";
import { MetricCard } from "@/components/shared/metric-card";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime, formatDate } from "@/lib/utils";
import type { ProjectSummary } from "@/types/api";

export default function ProjectsPage() {
  const { owner, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateProjectsView />;
  }

  if (!owner) return <NoRepoSelected />;

  return <ProjectsContent org={owner} />;
}

function AggregateProjectsView() {
  const router = useRouter();
  const [includeClosed, setIncludeClosed] = useState(false);

  const { data, error, isLoading } = useAggregateProjects(
    includeClosed ? { include_closed: true } : undefined,
  );

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const projects = data?.data || [];
  const openProjects = projects.filter((p) => !p.closed);
  const totalItems = projects.reduce((sum, p) => sum + p.total_items, 0);

  return (
    <ProjectsView
      title="Projects"
      subtitle="All Organizations"
      projects={projects}
      openCount={openProjects.length}
      totalItems={totalItems}
      includeClosed={includeClosed}
      onIncludeClosedChange={setIncludeClosed}
      onRowClick={(project) => router.push(`/projects/${project.owner}/${project.number}`)}
    />
  );
}

function ProjectsContent({ org }: { org: string }) {
  const router = useRouter();
  const [includeClosed, setIncludeClosed] = useState(false);

  const { data, error, isLoading } = useProjects(
    org,
    includeClosed ? { include_closed: true } : undefined,
  );

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const projects = data?.data || [];
  const openProjects = projects.filter((p) => !p.closed);
  const totalItems = projects.reduce((sum, p) => sum + p.total_items, 0);

  return (
    <ProjectsView
      title="Projects"
      subtitle={org}
      projects={projects}
      openCount={openProjects.length}
      totalItems={totalItems}
      includeClosed={includeClosed}
      onIncludeClosedChange={setIncludeClosed}
      onRowClick={(project) => router.push(`/projects/${project.owner}/${project.number}`)}
    />
  );
}

interface ProjectsViewProps {
  title: string;
  subtitle: string;
  projects: ProjectSummary[];
  openCount: number;
  totalItems: number;
  includeClosed: boolean;
  onIncludeClosedChange: (value: boolean) => void;
  onRowClick: (project: ProjectSummary) => void;
}

function ProjectsView({
  title,
  subtitle,
  projects,
  openCount,
  totalItems,
  includeClosed,
  onIncludeClosedChange,
  onRowClick,
}: ProjectsViewProps) {
  const columns: Column<ProjectSummary>[] = [
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
          {item.description && (
            <p className="mt-0.5 truncate text-xs text-gray-500 dark:text-gray-400">
              {item.description}
            </p>
          )}
        </div>
      ),
    },
    {
      key: "owner",
      header: "Owner",
      render: (item) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {item.owner}
        </span>
      ),
    },
    {
      key: "total_items",
      header: "Items",
      sortable: true,
      className: "w-20",
      render: (item) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {item.total_items}
        </span>
      ),
    },
    {
      key: "closed",
      header: "Status",
      render: (item) => (
        <span
          className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
            item.closed
              ? "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
              : "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
          }`}
        >
          {item.closed ? "Closed" : "Open"}
        </span>
      ),
    },
    {
      key: "synced_at",
      header: "Last Synced",
      sortable: true,
      render: (item) => (
        <span className="text-sm text-gray-500" title={formatDate(item.synced_at)}>
          {formatRelativeTime(item.synced_at)}
        </span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">{title}</h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
      </div>

      <FilterPanel
        filters={[
          {
            key: "status",
            label: "Status",
            value: includeClosed ? "all" : "open",
            onChange: (v) => onIncludeClosedChange(v === "all"),
            options: [
              { label: "Open Only", value: "open" },
              { label: "Include Closed", value: "all" },
            ],
          },
        ]}
        onClear={() => onIncludeClosedChange(false)}
      />

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <MetricCard title="Total Projects" value={projects.length} />
        <MetricCard title="Open Projects" value={openCount} />
        <MetricCard title="Total Items" value={totalItems} />
      </div>

      <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
        <DataTable
          columns={columns}
          data={projects}
          keyExtractor={(p) => `${p.owner}-${p.number}`}
          onRowClick={onRowClick}
          emptyMessage="No projects found"
        />
      </div>
    </div>
  );
}
