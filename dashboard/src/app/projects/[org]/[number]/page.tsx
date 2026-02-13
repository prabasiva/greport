"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useProjectDetail, useProjectItems, useProjectMetrics } from "@/hooks/use-api";
import { DataTable, type Column } from "@/components/shared/data-table";
import { Pagination } from "@/components/shared/pagination";
import { MetricCard } from "@/components/shared/metric-card";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay } from "@/components/shared/error-display";
import { PieChartComponent } from "@/components/charts/pie-chart-component";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import type { ProjectItemResponse, ProjectFieldSummary } from "@/types/api";

// ---------------------------------------------------------------------------
// Field value helpers
// ---------------------------------------------------------------------------

interface FieldValue {
  field_name: string;
  type: string;
  value?: string;
  name?: string;
  option_id?: string;
  title?: string;
  start_date?: string;
  duration?: number;
  iteration_id?: string;
}

interface SelectOption {
  id: string;
  name: string;
  color?: string;
  description?: string;
}

function getFieldValue(item: ProjectItemResponse, fieldName: string): FieldValue | undefined {
  if (!Array.isArray(item.field_values)) return undefined;
  return (item.field_values as FieldValue[]).find((fv) => fv.field_name === fieldName);
}

function getFieldDisplayValue(fv: FieldValue | undefined): string {
  if (!fv) return "";
  if (fv.type === "single_select") return fv.name || "";
  if (fv.type === "iteration") return fv.title || "";
  if (fv.type === "number") return fv.value ?? "";
  if (fv.type === "date") return fv.value ?? "";
  return fv.value ?? "";
}

function getStatusForItem(item: ProjectItemResponse): string {
  const fv = getFieldValue(item, "Status");
  if (fv?.type === "single_select" && fv.name) return fv.name;
  return "No Status";
}

const STATUS_COLORS: Record<string, { bg: string; border: string; dot: string }> = {
  GREEN: { bg: "bg-green-50 dark:bg-green-950", border: "border-green-300 dark:border-green-700", dot: "bg-green-500" },
  YELLOW: { bg: "bg-yellow-50 dark:bg-yellow-950", border: "border-yellow-300 dark:border-yellow-700", dot: "bg-yellow-500" },
  PURPLE: { bg: "bg-purple-50 dark:bg-purple-950", border: "border-purple-300 dark:border-purple-700", dot: "bg-purple-500" },
  RED: { bg: "bg-red-50 dark:bg-red-950", border: "border-red-300 dark:border-red-700", dot: "bg-red-500" },
  ORANGE: { bg: "bg-orange-50 dark:bg-orange-950", border: "border-orange-300 dark:border-orange-700", dot: "bg-orange-500" },
  BLUE: { bg: "bg-blue-50 dark:bg-blue-950", border: "border-blue-300 dark:border-blue-700", dot: "bg-blue-500" },
  PINK: { bg: "bg-pink-50 dark:bg-pink-950", border: "border-pink-300 dark:border-pink-700", dot: "bg-pink-500" },
  GRAY: { bg: "bg-gray-50 dark:bg-gray-900", border: "border-gray-300 dark:border-gray-700", dot: "bg-gray-500" },
};

const DEFAULT_STATUS_COLOR = STATUS_COLORS.GRAY;

function getStatusColumns(fields: ProjectFieldSummary[]): { name: string; color: string }[] {
  const statusField = fields.find((f) => f.name === "Status" && f.field_type === "single_select");
  if (!statusField || !Array.isArray(statusField.config_json)) return [];
  return (statusField.config_json as SelectOption[]).map((opt) => ({
    name: opt.name,
    color: opt.color || "GRAY",
  }));
}

// ---------------------------------------------------------------------------
// Main page component
// ---------------------------------------------------------------------------

export default function ProjectDetailPage() {
  const params = useParams();
  const org = params.org as string;
  const number = params.number ? Number(params.number) : null;

  const [viewMode, setViewMode] = useState<"board" | "table">("board");
  const [contentType, setContentType] = useState("all");
  const [stateFilter, setStateFilter] = useState("all");
  const [page, setPage] = useState(1);
  const perPage = 100;

  const itemParams: { content_type?: string; state?: string; page?: number; per_page?: number } = {
    page,
    per_page: perPage,
  };
  if (contentType !== "all") itemParams.content_type = contentType;
  if (stateFilter !== "all") itemParams.state = stateFilter;

  const { data: detailData, error: detailError, isLoading: detailLoading } = useProjectDetail(org, number);
  const { data: itemsData, error: itemsError, isLoading: itemsLoading } = useProjectItems(org, number, itemParams);
  const { data: metricsData } = useProjectMetrics(org, number);

  if (detailLoading) return <PageLoading />;
  if (detailError) return <ErrorDisplay message={detailError.message} />;

  const project = detailData?.data;
  if (!project) return <ErrorDisplay message="Project not found" />;

  const items = itemsData?.data || [];
  const meta = itemsData?.meta;
  const metrics = metricsData?.data;

  const statusColumns = getStatusColumns(project.fields);
  const hasStatusField = statusColumns.length > 0;

  // Extra fields worth showing (non-built-in, non-Status)
  const extraFields = project.fields.filter(
    (f) => f.field_type !== "built_in" && f.name !== "Status",
  );

  return (
    <div className="space-y-6">
      {/* Back link */}
      <Link
        href="/projects"
        className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
      >
        <svg className="mr-1 h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
        </svg>
        All Projects
      </Link>

      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-3">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
              {project.title}
            </h2>
            <span
              className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                project.closed
                  ? "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
                  : "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
              }`}
            >
              {project.closed ? "Closed" : "Open"}
            </span>
          </div>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            {project.owner}
          </p>
          {project.description && (
            <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">{project.description}</p>
          )}
        </div>
        <a
          href={project.url}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
        >
          Open on GitHub
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" />
          </svg>
        </a>
      </div>

      {/* Metric cards */}
      {metrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard title="Total Items" value={metrics.total_items} />
          {metrics.by_status.slice(0, 3).map((s) => (
            <MetricCard key={s.status} title={s.status || "No Status"} value={s.count} />
          ))}
        </div>
      )}

      {/* View toggle + filters */}
      <div className="flex flex-wrap items-center gap-4">
        {hasStatusField && (
          <div className="inline-flex rounded-lg border border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-900">
            <button
              onClick={() => setViewMode("board")}
              className={`inline-flex items-center gap-1.5 rounded-l-lg px-3 py-1.5 text-sm font-medium transition-colors ${
                viewMode === "board"
                  ? "bg-blue-50 text-blue-700 dark:bg-blue-950 dark:text-blue-300"
                  : "text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-800"
              }`}
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 4.5v15m6-15v15m-10.875 0h15.75c.621 0 1.125-.504 1.125-1.125V5.625c0-.621-.504-1.125-1.125-1.125H4.125C3.504 4.5 3 5.004 3 5.625v12.75c0 .621.504 1.125 1.125 1.125Z" />
              </svg>
              Board
            </button>
            <button
              onClick={() => setViewMode("table")}
              className={`inline-flex items-center gap-1.5 rounded-r-lg px-3 py-1.5 text-sm font-medium transition-colors ${
                viewMode === "table"
                  ? "bg-blue-50 text-blue-700 dark:bg-blue-950 dark:text-blue-300"
                  : "text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-800"
              }`}
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" d="M3.375 19.5h17.25m-17.25 0a1.125 1.125 0 0 1-1.125-1.125M3.375 19.5h7.5c.621 0 1.125-.504 1.125-1.125m-9.75 0V5.625m0 12.75v-1.5c0-.621.504-1.125 1.125-1.125m18.375 2.625V5.625m0 12.75c0 .621-.504 1.125-1.125 1.125m1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125m0 3.75h-7.5A1.125 1.125 0 0 1 12 18.375m9.75-12.75c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125m19.5 0v1.5c0 .621-.504 1.125-1.125 1.125M2.25 5.625v1.5c0 .621.504 1.125 1.125 1.125m0 0h17.25m-17.25 0h7.5c.621 0 1.125.504 1.125 1.125M3.375 8.25c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125m17.25-3.75h-7.5c-.621 0-1.125.504-1.125 1.125m8.625-1.125c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125m-17.25 0h7.5m-7.5 0c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125M12 10.875v-1.5m0 1.5c0 .621-.504 1.125-1.125 1.125M12 10.875c0 .621.504 1.125 1.125 1.125m-2.25 0c.621 0 1.125.504 1.125 1.125M13.125 12h7.5m-7.5 0c-.621 0-1.125.504-1.125 1.125M20.625 12c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125m-17.25 0h7.5M12 14.625v-1.5m0 1.5c0 .621-.504 1.125-1.125 1.125M12 14.625c0 .621.504 1.125 1.125 1.125m-2.25 0c.621 0 1.125.504 1.125 1.125m0 0v1.5c0 .621-.504 1.125-1.125 1.125" />
              </svg>
              Table
            </button>
          </div>
        )}

        <div className="flex items-center gap-2">
          <select
            value={contentType}
            onChange={(e) => { setContentType(e.target.value); setPage(1); }}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300"
          >
            <option value="all">All Types</option>
            <option value="Issue">Issue</option>
            <option value="PullRequest">Pull Request</option>
            <option value="DraftIssue">Draft Issue</option>
          </select>
          <select
            value={stateFilter}
            onChange={(e) => { setStateFilter(e.target.value); setPage(1); }}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-300"
          >
            <option value="all">All States</option>
            <option value="open">Open</option>
            <option value="closed">Closed</option>
            <option value="merged">Merged</option>
          </select>
          {(contentType !== "all" || stateFilter !== "all") && (
            <button
              onClick={() => { setContentType("all"); setStateFilter("all"); setPage(1); }}
              className="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Board or Table view */}
      {itemsLoading ? (
        <div className="flex items-center justify-center py-12">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-blue-500 border-t-transparent" />
        </div>
      ) : itemsError ? (
        <ErrorDisplay message={itemsError.message} />
      ) : viewMode === "board" && hasStatusField ? (
        <BoardView items={items} statusColumns={statusColumns} extraFields={extraFields} />
      ) : (
        <TableView items={items} meta={meta} extraFields={extraFields} onPageChange={setPage} />
      )}

      {/* Pagination for board view */}
      {viewMode === "board" && meta && meta.total_pages > 1 && (
        <Pagination meta={meta} onPageChange={setPage} />
      )}

      {/* Charts */}
      {metrics && (metrics.by_status.length > 0 || metrics.by_content_type.length > 0) && (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {metrics.by_status.length > 0 && (
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
                Status Distribution
              </h3>
              <PieChartComponent
                data={metrics.by_status.map((s) => ({ name: s.status || "No Status", value: s.count }))}
              />
            </div>
          )}
          {metrics.by_content_type.length > 0 && (
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
                Content Type Distribution
              </h3>
              <BarChartComponent
                data={metrics.by_content_type.map((c) => ({ name: c.content_type, value: c.count }))}
                layout="horizontal"
                color="#8b5cf6"
              />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Board View (Kanban)
// ---------------------------------------------------------------------------

function BoardView({
  items,
  statusColumns,
  extraFields,
}: {
  items: ProjectItemResponse[];
  statusColumns: { name: string; color: string }[];
  extraFields: ProjectFieldSummary[];
}) {
  const grouped = useMemo(() => {
    const groups: Record<string, ProjectItemResponse[]> = {};
    // Initialize columns in order
    for (const col of statusColumns) {
      groups[col.name] = [];
    }
    groups["No Status"] = [];

    for (const item of items) {
      const status = getStatusForItem(item);
      if (groups[status]) {
        groups[status].push(item);
      } else {
        groups["No Status"].push(item);
      }
    }
    return groups;
  }, [items, statusColumns]);

  // Only show "No Status" column if it has items
  const columnsToShow = [
    ...statusColumns,
    ...(grouped["No Status"].length > 0 ? [{ name: "No Status", color: "GRAY" }] : []),
  ];

  if (items.length === 0) {
    return (
      <div className="rounded-lg border border-gray-200 bg-white p-12 text-center dark:border-gray-800 dark:bg-gray-900">
        <p className="text-sm text-gray-500 dark:text-gray-400">No items in this project</p>
      </div>
    );
  }

  return (
    <div className="flex gap-4 overflow-x-auto pb-4">
      {columnsToShow.map((col) => {
        const colItems = grouped[col.name] || [];
        const colors = STATUS_COLORS[col.color] || DEFAULT_STATUS_COLOR;
        return (
          <div key={col.name} className="flex w-72 flex-shrink-0 flex-col">
            {/* Column header */}
            <div className={`flex items-center gap-2 rounded-t-lg border ${colors.border} ${colors.bg} px-3 py-2`}>
              <span className={`h-2.5 w-2.5 rounded-full ${colors.dot}`} />
              <span className="text-sm font-semibold text-gray-800 dark:text-gray-200">
                {col.name}
              </span>
              <span className="ml-auto text-xs text-gray-500 dark:text-gray-400">
                {colItems.length}
              </span>
            </div>
            {/* Cards */}
            <div className="flex flex-1 flex-col gap-2 rounded-b-lg border border-t-0 border-gray-200 bg-gray-50 p-2 dark:border-gray-700 dark:bg-gray-900/50">
              {colItems.length === 0 ? (
                <p className="py-4 text-center text-xs text-gray-400 dark:text-gray-600">
                  No items
                </p>
              ) : (
                colItems.map((item) => (
                  <ItemCard key={item.node_id} item={item} extraFields={extraFields} />
                ))
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Item Card (for board view)
// ---------------------------------------------------------------------------

function ItemCard({
  item,
  extraFields,
}: {
  item: ProjectItemResponse;
  extraFields: ProjectFieldSummary[];
}) {
  const typeColors: Record<string, string> = {
    issue: "text-blue-600 dark:text-blue-400",
    pullrequest: "text-purple-600 dark:text-purple-400",
    draftissue: "text-gray-500 dark:text-gray-400",
  };

  const typeIcons: Record<string, string> = {
    issue: "M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
    pullrequest: "M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5",
    draftissue: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z",
  };

  const ct = item.content_type.toLowerCase();
  const stateColor = item.content_state === "OPEN" || item.content_state === "open"
    ? "text-green-600 dark:text-green-400"
    : item.content_state === "MERGED" || item.content_state === "merged"
      ? "text-purple-600 dark:text-purple-400"
      : "text-red-600 dark:text-red-400";

  // Collect displayable extra field values
  const fieldBadges: { name: string; value: string }[] = [];
  for (const f of extraFields) {
    const fv = getFieldValue(item, f.name);
    const display = getFieldDisplayValue(fv);
    if (display) fieldBadges.push({ name: f.name, value: display });
  }

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-3 shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
      {/* Title row */}
      <div className="flex items-start gap-2">
        <svg
          className={`mt-0.5 h-4 w-4 flex-shrink-0 ${typeColors[ct] || "text-gray-500"}`}
          fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d={typeIcons[ct] || typeIcons.issue} />
        </svg>
        <div className="min-w-0 flex-1">
          {item.content_url ? (
            <a
              href={item.content_url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm font-medium text-gray-900 hover:text-blue-600 dark:text-white dark:hover:text-blue-400"
            >
              {item.content_title}
            </a>
          ) : (
            <span className="text-sm font-medium text-gray-900 dark:text-white">
              {item.content_title}
            </span>
          )}
        </div>
      </div>

      {/* Meta row */}
      <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-500 dark:text-gray-400">
        {item.content_number && (
          <span className="font-mono">#{item.content_number}</span>
        )}
        {item.content_state && (
          <span className={`font-medium ${stateColor}`}>
            {item.content_state.charAt(0) + item.content_state.slice(1).toLowerCase()}
          </span>
        )}
        {item.content_repository && (
          <span>{item.content_repository.split("/").pop()}</span>
        )}
      </div>

      {/* Field value badges */}
      {fieldBadges.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {fieldBadges.map((fb) => (
            <span
              key={fb.name}
              className="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-700 dark:text-gray-300"
              title={fb.name}
            >
              {fb.value}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Table View
// ---------------------------------------------------------------------------

function TableView({
  items,
  meta,
  extraFields,
  onPageChange,
}: {
  items: ProjectItemResponse[];
  meta?: { page: number; per_page: number; total: number; total_pages: number };
  extraFields: ProjectFieldSummary[];
  onPageChange: (page: number) => void;
}) {
  const columns: Column<ProjectItemResponse>[] = [
    {
      key: "content_number",
      header: "#",
      sortable: true,
      className: "w-16",
      render: (item) => (
        <span className="font-mono text-xs text-gray-500">
          {item.content_number ? `#${item.content_number}` : "-"}
        </span>
      ),
    },
    {
      key: "content_title",
      header: "Title",
      render: (item) => (
        <div className="max-w-md">
          {item.content_url ? (
            <a
              href={item.content_url}
              target="_blank"
              rel="noopener noreferrer"
              className="truncate font-medium text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
              onClick={(e) => e.stopPropagation()}
            >
              {item.content_title}
            </a>
          ) : (
            <p className="truncate font-medium text-gray-900 dark:text-white">
              {item.content_title}
            </p>
          )}
        </div>
      ),
    },
    {
      key: "status",
      header: "Status",
      render: (item) => {
        const status = getStatusForItem(item);
        if (status === "No Status") return <span className="text-sm text-gray-400">-</span>;
        return (
          <span className="inline-flex items-center rounded-full bg-gray-100 px-2 py-1 text-xs font-medium text-gray-700 dark:bg-gray-700 dark:text-gray-300">
            {status}
          </span>
        );
      },
    },
    {
      key: "content_type",
      header: "Type",
      render: (item) => {
        const typeColors: Record<string, string> = {
          issue: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
          pullrequest: "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300",
          draftissue: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300",
        };
        const ct = item.content_type.toLowerCase();
        return (
          <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${typeColors[ct] || "bg-gray-100 text-gray-700"}`}>
            {item.content_type}
          </span>
        );
      },
    },
    {
      key: "content_state",
      header: "State",
      render: (item) => {
        if (!item.content_state) return <span className="text-sm text-gray-400">-</span>;
        const s = item.content_state.toLowerCase();
        const stateColors: Record<string, string> = {
          open: "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
          closed: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300",
          merged: "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300",
        };
        return (
          <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${stateColors[s] || "bg-gray-100 text-gray-700"}`}>
            {item.content_state}
          </span>
        );
      },
    },
    {
      key: "content_repository",
      header: "Repository",
      render: (item) => (
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {item.content_repository ? item.content_repository.split("/").pop() || item.content_repository : "-"}
        </span>
      ),
    },
    // Dynamic columns for extra fields
    ...extraFields.map((f) => ({
      key: `field_${f.name}`,
      header: f.name,
      render: (item: ProjectItemResponse) => {
        const fv = getFieldValue(item, f.name);
        const display = getFieldDisplayValue(fv);
        if (!display) return <span className="text-sm text-gray-400">-</span>;
        return (
          <span className="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-700 dark:bg-gray-700 dark:text-gray-300">
            {display}
          </span>
        );
      },
    })),
  ];

  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 shadow-sm dark:border-gray-800">
      <DataTable
        columns={columns}
        data={items}
        keyExtractor={(item) => item.node_id}
        emptyMessage="No items found matching the filters"
      />
      {meta && <Pagination meta={meta} onPageChange={onPageChange} />}
    </div>
  );
}
