"use client";

import { useRepo } from "@/hooks/use-repo";
import { useIssueMetrics, useVelocity, useStaleIssues, useAggregateIssueMetrics, useAggregateVelocity } from "@/hooks/use-api";
import { MetricCard } from "@/components/shared/metric-card";
import { TrendChart } from "@/components/charts/trend-chart";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatHours } from "@/lib/utils";
import type { Trend } from "@/types/api";

export default function DashboardPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateDashboard />;
  }

  if (!owner || !repo) {
    return <NoRepoSelected />;
  }

  return <DashboardContent owner={owner} repo={repo} />;
}

function AggregateDashboard() {
  const { data: metricsData, error: metricsError, isLoading: metricsLoading } = useAggregateIssueMetrics();
  const { data: velocityData, error: velocityError, isLoading: velocityLoading } = useAggregateVelocity({ period: "week", last: 12 });

  if (metricsLoading || velocityLoading) return <PageLoading />;
  if (metricsError) return <ErrorDisplay message={metricsError.message} />;
  if (velocityError) return <ErrorDisplay message={velocityError.message} />;

  const metrics = metricsData?.data;
  const velocity = velocityData?.data;

  if (!metrics) return <PageLoading />;

  const labelData = Object.entries(metrics.by_label)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10)
    .map(([name, value]) => ({ name, value }));

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          All Repositories
        </p>
      </div>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Issues"
          value={metrics.totals.total}
          subtitle={`${metrics.totals.open} open / ${metrics.totals.closed} closed`}
        />
        <MetricCard
          title="Open Issues"
          value={metrics.totals.open}
          trend={velocity?.trend as Trend | undefined}
          trendValue={velocity?.trend || ""}
        />
        <MetricCard
          title="Stale Issues"
          value={metrics.totals.stale_count}
          subtitle="Inactive for 30+ days"
        />
        <MetricCard
          title="Repositories"
          value={metrics.by_repository.length}
          subtitle="Tracked repositories"
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Issues by Repository */}
        {metrics.by_repository.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Issues by Repository
            </h3>
            <BarChartComponent
              data={metrics.by_repository.map((r) => ({
                name: r.repository.split("/").pop() || r.repository,
                value: r.total,
              }))}
            />
          </div>
        )}

        {/* Label Distribution */}
        {labelData.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Issues by Label
            </h3>
            <BarChartComponent data={labelData} />
          </div>
        )}
      </div>

      {/* Velocity Summary */}
      {velocity && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Velocity Summary (Weekly)
          </h3>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <div>
              <BarChartComponent
                data={velocity.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.avg_opened,
                }))}
                color="#ef4444"
              />
              <p className="mt-2 text-center text-sm text-gray-500">Avg Opened / Week</p>
            </div>
            <div>
              <BarChartComponent
                data={velocity.by_repository.map((r) => ({
                  name: r.repository.split("/").pop() || r.repository,
                  value: r.avg_closed,
                }))}
                color="#22c55e"
              />
              <p className="mt-2 text-center text-sm text-gray-500">Avg Closed / Week</p>
            </div>
          </div>
          <div className="mt-4 flex gap-6 text-sm text-gray-500 dark:text-gray-400">
            <span>Combined Avg Opened: {velocity.combined_avg_opened.toFixed(1)}/week</span>
            <span>Combined Avg Closed: {velocity.combined_avg_closed.toFixed(1)}/week</span>
            <span>Trend: {velocity.trend}</span>
          </div>
        </div>
      )}

      {/* Age Distribution */}
      {metrics.age_distribution.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Issue Age Distribution
          </h3>
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

      {/* Per-Repo Breakdown Table */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
          Per-Repository Breakdown
        </h3>
        <div className="overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="px-4 py-2 text-left font-medium text-gray-500">Repository</th>
                <th className="px-4 py-2 text-right font-medium text-gray-500">Total</th>
                <th className="px-4 py-2 text-right font-medium text-gray-500">Open</th>
                <th className="px-4 py-2 text-right font-medium text-gray-500">Closed</th>
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
                  <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.stale_count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function DashboardContent({ owner, repo }: { owner: string; repo: string }) {
  const { data: metricsData, error: metricsError, isLoading: metricsLoading } = useIssueMetrics(owner, repo);
  const { data: velocityData, error: velocityError, isLoading: velocityLoading } = useVelocity(owner, repo, { period: "week", last: 12 });
  const { data: staleData } = useStaleIssues(owner, repo, 30);

  if (metricsLoading || velocityLoading) return <PageLoading />;
  if (metricsError) return <ErrorDisplay message={metricsError.message} />;
  if (velocityError) return <ErrorDisplay message={velocityError.message} />;

  const metrics = metricsData?.data;
  const velocity = velocityData?.data;
  const staleIssues = staleData?.data;

  if (!metrics) return <PageLoading />;

  const labelData = Object.entries(metrics.by_label)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10)
    .map(([name, value]) => ({ name, value }));

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {owner}/{repo}
        </p>
      </div>

      {/* Metric Cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Issues"
          value={metrics.total}
          subtitle={`${metrics.open} open / ${metrics.closed} closed`}
        />
        <MetricCard
          title="Open Issues"
          value={metrics.open}
          trend={velocity?.trend}
          trendValue={velocity?.trend || ""}
        />
        <MetricCard
          title="Avg Time to Close"
          value={metrics.avg_time_to_close_hours ? formatHours(metrics.avg_time_to_close_hours) : "N/A"}
          subtitle={metrics.median_time_to_close_hours ? `Median: ${formatHours(metrics.median_time_to_close_hours)}` : undefined}
        />
        <MetricCard
          title="Stale Issues"
          value={metrics.stale_count}
          subtitle="Inactive for 30+ days"
        />
      </div>

      {/* Stale Issues Alert */}
      {staleIssues && staleIssues.length > 0 && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 p-4 dark:border-amber-900 dark:bg-amber-950">
          <div className="flex items-start gap-3">
            <svg className="h-5 w-5 shrink-0 text-amber-500 mt-0.5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
            <div>
              <h3 className="text-sm font-medium text-amber-800 dark:text-amber-200">
                {staleIssues.length} stale issue{staleIssues.length !== 1 ? "s" : ""} detected
              </h3>
              <p className="mt-1 text-sm text-amber-700 dark:text-amber-300">
                These issues have had no activity for over 30 days. Consider triaging or closing them.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Charts Row */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Velocity Trend */}
        {velocity && velocity.data_points.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Issue Velocity (Weekly)
            </h3>
            <TrendChart data={velocity.data_points} />
            <div className="mt-4 flex gap-6 text-sm text-gray-500 dark:text-gray-400">
              <span>Avg Opened: {velocity.avg_opened.toFixed(1)}/week</span>
              <span>Avg Closed: {velocity.avg_closed.toFixed(1)}/week</span>
            </div>
          </div>
        )}

        {/* Label Distribution */}
        {labelData.length > 0 && (
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Issues by Label
            </h3>
            <BarChartComponent data={labelData} />
          </div>
        )}
      </div>

      {/* Age Distribution */}
      {metrics.age_distribution.buckets.length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Issue Age Distribution
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

      {/* Assignee Distribution */}
      {Object.keys(metrics.by_assignee).length > 0 && (
        <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
          <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
            Issues by Assignee
          </h3>
          <BarChartComponent
            data={Object.entries(metrics.by_assignee)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 10)
              .map(([name, value]) => ({ name, value }))}
            color="#22c55e"
          />
        </div>
      )}
    </div>
  );
}
