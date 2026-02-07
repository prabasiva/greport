"use client";

import { useState } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useVelocity, useBurndown, useAggregateVelocity } from "@/hooks/use-api";
import { TrendChart } from "@/components/charts/trend-chart";
import { BurndownChart } from "@/components/charts/burndown-chart";
import { BarChartComponent } from "@/components/charts/bar-chart-component";
import { MetricCard } from "@/components/shared/metric-card";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { formatDate } from "@/lib/utils";
import type { Period, Trend } from "@/types/api";


export default function VelocityPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateVelocityView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;
  return <VelocityContent owner={owner} repo={repo} />;
}

function AggregateVelocityView() {
  const [period, setPeriod] = useState<string>("week");
  const [last, setLast] = useState(12);

  const { data, error, isLoading } = useAggregateVelocity({ period, last });

  if (isLoading) return <PageLoading />;
  if (error) return <ErrorDisplay message={error.message} />;

  const velocity = data?.data;
  if (!velocity) return <PageLoading />;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Velocity
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          All Repositories
        </p>
      </div>

      {/* Period Selector */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Period
          </label>
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value="day">Daily</option>
            <option value="week">Weekly</option>
            <option value="month">Monthly</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Last
          </label>
          <select
            value={last}
            onChange={(e) => setLast(Number(e.target.value))}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value={4}>4</option>
            <option value={8}>8</option>
            <option value={12}>12</option>
            <option value={24}>24</option>
            <option value={52}>52</option>
          </select>
        </div>
      </div>

      {/* Summary Metrics */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <MetricCard
          title="Combined Avg Opened"
          value={`${velocity.combined_avg_opened.toFixed(1)}/${velocity.period}`}
          trend={velocity.trend as Trend}
          trendValue={velocity.trend}
        />
        <MetricCard
          title="Combined Avg Closed"
          value={`${velocity.combined_avg_closed.toFixed(1)}/${velocity.period}`}
        />
        <MetricCard
          title="Trend"
          value={velocity.trend}
        />
      </div>

      {/* Per-Repository Velocity */}
      {velocity.by_repository.length > 0 && (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Avg Opened per {velocity.period}
            </h3>
            <BarChartComponent
              data={velocity.by_repository.map((r) => ({
                name: r.repository.split("/").pop() || r.repository,
                value: r.avg_opened,
              }))}
              color="#ef4444"
            />
          </div>
          <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
              Avg Closed per {velocity.period}
            </h3>
            <BarChartComponent
              data={velocity.by_repository.map((r) => ({
                name: r.repository.split("/").pop() || r.repository,
                value: r.avg_closed,
              }))}
              color="#22c55e"
            />
          </div>
        </div>
      )}

      {/* Per-Repo Table */}
      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
        <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
          Per-Repository Breakdown
        </h3>
        <div className="overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="px-4 py-2 text-left font-medium text-gray-500">Repository</th>
                <th className="px-4 py-2 text-right font-medium text-gray-500">Avg Opened</th>
                <th className="px-4 py-2 text-right font-medium text-gray-500">Avg Closed</th>
              </tr>
            </thead>
            <tbody>
              {velocity.by_repository.map((r) => (
                <tr key={r.repository} className="border-b border-gray-100 dark:border-gray-800">
                  <td className="px-4 py-2 font-medium text-gray-900 dark:text-white">{r.repository}</td>
                  <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.avg_opened.toFixed(1)}</td>
                  <td className="px-4 py-2 text-right text-gray-600 dark:text-gray-300">{r.avg_closed.toFixed(1)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function VelocityContent({ owner, repo }: { owner: string; repo: string }) {
  const [period, setPeriod] = useState<Period>("week");
  const [last, setLast] = useState(12);
  const [milestone, setMilestone] = useState("");

  const { data: velocityData, error: velocityError, isLoading: velocityLoading } = useVelocity(owner, repo, { period, last });
  const { data: burndownData, error: burndownError, isLoading: burndownLoading } = useBurndown(owner, repo, milestone || null);

  if (velocityLoading) return <PageLoading />;
  if (velocityError) return <ErrorDisplay message={velocityError.message} />;

  const velocity = velocityData?.data;
  const burndown = burndownData?.data;

  function trendLabel(trend: Trend): string {
    switch (trend) {
      case "increasing": return "Issues growing faster than being closed";
      case "decreasing": return "Issues are being closed faster than opened";
      case "stable": return "Issue rate is stable";
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
          Velocity & Burndown
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {owner}/{repo}
        </p>
      </div>

      {/* Period Selector */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Period
          </label>
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value as Period)}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value="day">Daily</option>
            <option value="week">Weekly</option>
            <option value="month">Monthly</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Last
          </label>
          <select
            value={last}
            onChange={(e) => setLast(Number(e.target.value))}
            className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          >
            <option value={4}>4</option>
            <option value={8}>8</option>
            <option value={12}>12</option>
            <option value={24}>24</option>
            <option value={52}>52</option>
          </select>
        </div>
      </div>

      {/* Velocity Metrics */}
      {velocity && (
        <>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <MetricCard
              title="Avg Opened"
              value={`${velocity.avg_opened.toFixed(1)}/${period}`}
              trend={velocity.trend}
              trendValue={velocity.trend}
            />
            <MetricCard
              title="Avg Closed"
              value={`${velocity.avg_closed.toFixed(1)}/${period}`}
            />
            <MetricCard
              title="Trend"
              value={velocity.trend}
              subtitle={trendLabel(velocity.trend)}
            />
          </div>

          {velocity.data_points.length > 0 && (
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
                Issue Velocity
              </h3>
              <TrendChart data={velocity.data_points} height={400} />
            </div>
          )}

          {/* Cumulative Chart */}
          {velocity.data_points.length > 0 && (
            <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
              <h3 className="mb-4 text-lg font-semibold text-gray-900 dark:text-white">
                Net Change Over Time
              </h3>
              <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-200 dark:border-gray-700">
                      <th className="px-3 py-2 text-left text-xs font-medium text-gray-500">Period</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500">Opened</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500">Closed</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500">Net</th>
                      <th className="px-3 py-2 text-right text-xs font-medium text-gray-500">Cumulative</th>
                    </tr>
                  </thead>
                  <tbody>
                    {velocity.data_points.map((dp) => (
                      <tr key={dp.period_start} className="border-b border-gray-100 dark:border-gray-800">
                        <td className="px-3 py-2 text-gray-700 dark:text-gray-300">
                          {formatDate(dp.period_start)}
                        </td>
                        <td className="px-3 py-2 text-right text-red-600">{dp.opened}</td>
                        <td className="px-3 py-2 text-right text-green-600">{dp.closed}</td>
                        <td className={`px-3 py-2 text-right font-medium ${dp.net_change > 0 ? "text-red-600" : dp.net_change < 0 ? "text-green-600" : "text-gray-500"}`}>
                          {dp.net_change > 0 ? "+" : ""}{dp.net_change}
                        </td>
                        <td className="px-3 py-2 text-right text-gray-700 dark:text-gray-300">
                          {dp.cumulative_open}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </>
      )}

      {/* Burndown Section */}
      <div className="border-t border-gray-200 pt-6 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Burndown Chart
        </h3>
        <div className="mt-4 flex items-center gap-2">
          <input
            type="text"
            placeholder="Enter milestone name..."
            value={milestone}
            onChange={(e) => setMilestone(e.target.value)}
            className="rounded-md border border-gray-300 px-3 py-1.5 text-sm shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
          />
        </div>

        {burndownLoading && milestone && <PageLoading />}
        {burndownError && milestone && <ErrorDisplay message={burndownError.message} />}

        {burndown && (
          <div className="mt-4 rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900">
            <div className="mb-4 flex items-center justify-between">
              <div>
                <h4 className="font-medium text-gray-900 dark:text-white">
                  {burndown.milestone}
                </h4>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {burndown.total_issues} total issues
                  {burndown.projected_completion && ` - Projected completion: ${formatDate(burndown.projected_completion)}`}
                </p>
              </div>
            </div>
            <BurndownChart
              actual={burndown.data_points}
              ideal={burndown.ideal_burndown}
              height={400}
            />
          </div>
        )}
      </div>
    </div>
  );
}
