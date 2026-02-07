"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useRepos } from "@/hooks/use-repos";
import { useSla } from "@/hooks/use-api";
import { MetricCard } from "@/components/shared/metric-card";
import { DataTable, type Column } from "@/components/shared/data-table";
import { PieChartComponent } from "@/components/charts/pie-chart-component";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatRelativeTime } from "@/lib/utils";
import type { SlaIssue } from "@/types/api";

export default function SlaPage() {
  const { activeRepo, repos } = useRepos();

  if (!activeRepo) {
    return <AggregateSlaView repos={repos} />;
  }

  return <SlaContent owner={activeRepo.owner} repo={activeRepo.name} />;
}

function AggregateSlaView({ repos }: { repos: { owner: string; name: string; fullName: string }[] }) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          SLA Compliance
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          All repositories
        </p>
      </div>
      {repos.length === 0 ? (
        <div className="py-12 text-center text-sm text-gray-500">
          No repositories tracked. Add a repository to see SLA reports.
        </div>
      ) : (
        repos.map((repo) => (
          <RepoSlaSection key={repo.fullName} owner={repo.owner} name={repo.name} fullName={repo.fullName} />
        ))
      )}
    </div>
  );
}

function RepoSlaSection({ owner, name, fullName }: { owner: string; name: string; fullName: string }) {
  return (
    <div className="space-y-3">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white border-b border-gray-200 dark:border-gray-700 pb-2">
        {fullName}
      </h3>
      <SlaContent owner={owner} repo={name} />
    </div>
  );
}

function SlaContent({ owner, repo }: { owner: string; repo: string }) {
  const [responseHours, setResponseHours] = useState(24);
  const [resolutionHours, setResolutionHours] = useState(168);

  const { data, error, isLoading } = useSla(owner, repo, {
    response_hours: responseHours,
    resolution_hours: resolutionHours,
  });

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const sla = data?.data;
  if (!sla) return <PageLoading />;

  const summary = sla.summary;

  const complianceData = [
    { name: "Within SLA", value: summary.within_sla },
    { name: "Response Breached", value: summary.response_breached },
    { name: "Resolution Breached", value: summary.resolution_breached },
    { name: "At Risk", value: summary.at_risk },
  ].filter((d) => d.value > 0);

  const issueColumns: Column<SlaIssue>[] = [
    {
      key: "number",
      header: "#",
      className: "w-16",
      render: (i) => <span className="font-mono text-xs">#{i.number}</span>,
    },
    {
      key: "title",
      header: "Title",
      render: (i) => (
        <div className="max-w-sm">
          <p className="truncate font-medium text-gray-900 dark:text-white">
            {i.title}
          </p>
          <div className="mt-1 flex gap-1">
            {i.labels.map((label) => (
              <span
                key={label}
                className="rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-800 dark:text-gray-400"
              >
                {label}
              </span>
            ))}
          </div>
        </div>
      ),
    },
    {
      key: "author",
      header: "Author",
      render: (i) => <span className="text-sm">{i.author}</span>,
    },
    {
      key: "age_hours",
      header: "Age",
      sortable: true,
      render: (i) => (
        <span className="text-sm">
          {i.age_hours < 24
            ? `${i.age_hours.toFixed(1)}h`
            : `${(i.age_hours / 24).toFixed(1)}d`}
        </span>
      ),
    },
    {
      key: "status",
      header: "Status",
      render: (i) => <SlaStatusBadge status={i.sla_status} />,
    },
    {
      key: "created_at",
      header: "Created",
      render: (i) => (
        <span className="text-sm text-gray-500">
          {formatRelativeTime(i.created_at)}
        </span>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          SLA Compliance
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {owner}/{repo}
        </p>
      </div>

      {/* SLA Configuration */}
      <div className="flex flex-wrap items-center gap-4 rounded-lg border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-gray-800/50">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Response SLA
          </label>
          <select
            value={responseHours}
            onChange={(e) => setResponseHours(Number(e.target.value))}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          >
            <option value={4}>4 hours</option>
            <option value={8}>8 hours</option>
            <option value={24}>24 hours</option>
            <option value={48}>48 hours</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Resolution SLA
          </label>
          <select
            value={resolutionHours}
            onChange={(e) => setResolutionHours(Number(e.target.value))}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          >
            <option value={24}>1 day</option>
            <option value={72}>3 days</option>
            <option value={168}>7 days</option>
            <option value={336}>14 days</option>
            <option value={720}>30 days</option>
          </select>
        </div>
      </div>

      {/* Summary Metrics */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Compliance Rate"
          value={`${summary.compliance_rate.toFixed(1)}%`}
          subtitle={`${summary.within_sla} of ${summary.total_open} within SLA`}
        />
        <MetricCard
          title="Response Breached"
          value={summary.response_breached}
          subtitle={`Over ${sla.config.response_time_hours}h without response`}
        />
        <MetricCard
          title="Resolution Breached"
          value={summary.resolution_breached}
          subtitle={`Over ${sla.config.resolution_time_hours}h open`}
        />
        <MetricCard
          title="At Risk"
          value={summary.at_risk}
          subtitle="Approaching SLA deadline"
        />
      </div>

      {/* Compliance Chart */}
      {complianceData.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            SLA Status Distribution
          </h3>
          <PieChartComponent data={complianceData} />
        </div>
      )}

      {/* Breaching Issues */}
      {sla.breaching_issues.length > 0 && (
        <div>
          <h3 className="mb-3 text-lg font-semibold text-red-600 dark:text-red-400">
            Breaching Issues ({sla.breaching_issues.length})
          </h3>
          <div className="overflow-hidden rounded-lg border border-red-200 shadow-sm dark:border-red-900">
            <DataTable
              columns={issueColumns}
              data={sla.breaching_issues}
              keyExtractor={(i) => i.number}
            />
          </div>
        </div>
      )}

      {/* At Risk Issues */}
      {sla.at_risk_issues.length > 0 && (
        <div>
          <h3 className="mb-3 text-lg font-semibold text-amber-600 dark:text-amber-400">
            At Risk Issues ({sla.at_risk_issues.length})
          </h3>
          <div className="overflow-hidden rounded-lg border border-amber-200 shadow-sm dark:border-amber-900">
            <DataTable
              columns={issueColumns}
              data={sla.at_risk_issues}
              keyExtractor={(i) => i.number}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function SlaStatusBadge({ status }: { status: SlaIssue["sla_status"] }) {
  switch (status.type) {
    case "Ok":
      return (
        <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-700 dark:bg-green-900 dark:text-green-300">
          OK
        </span>
      );
    case "AtRisk":
      return (
        <span className="rounded-full bg-amber-100 px-2 py-1 text-xs font-medium text-amber-700 dark:bg-amber-900 dark:text-amber-300">
          At Risk ({status.percent_elapsed.toFixed(0)}%)
        </span>
      );
    case "ResponseBreached":
      return (
        <span className="rounded-full bg-red-100 px-2 py-1 text-xs font-medium text-red-700 dark:bg-red-900 dark:text-red-300">
          Response +{status.hours_overdue.toFixed(1)}h
        </span>
      );
    case "ResolutionBreached":
      return (
        <span className="rounded-full bg-red-100 px-2 py-1 text-xs font-medium text-red-700 dark:bg-red-900 dark:text-red-300">
          Resolution +{status.hours_overdue.toFixed(1)}h
        </span>
      );
  }
}
