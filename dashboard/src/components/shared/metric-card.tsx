import { cn, formatNumber } from "@/lib/utils";
import type { Trend } from "@/types/api";

interface MetricCardProps {
  title: string;
  value: number | string;
  subtitle?: string;
  trend?: Trend;
  trendValue?: string;
  className?: string;
}

export function MetricCard({
  title,
  value,
  subtitle,
  trend,
  trendValue,
  className,
}: MetricCardProps) {
  const displayValue = typeof value === "number" ? formatNumber(value) : value;

  return (
    <div
      className={cn(
        "rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-gray-900",
        className,
      )}
    >
      <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
        {title}
      </p>
      <div className="mt-2 flex items-baseline gap-2">
        <p className="text-3xl font-semibold text-gray-900 dark:text-white">
          {displayValue}
        </p>
        {trend && trendValue && <TrendBadge trend={trend} value={trendValue} />}
      </div>
      {subtitle && (
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {subtitle}
        </p>
      )}
    </div>
  );
}

function TrendBadge({ trend, value }: { trend: Trend; value: string }) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
        trend === "increasing" &&
          "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
        trend === "decreasing" &&
          "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300",
        trend === "stable" &&
          "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300",
      )}
    >
      {trend === "increasing" && (
        <svg className="mr-0.5 h-3 w-3" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 19.5l15-15m0 0H8.25m11.25 0v11.25" />
        </svg>
      )}
      {trend === "decreasing" && (
        <svg className="mr-0.5 h-3 w-3" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 4.5l15 15m0 0V8.25m0 11.25H8.25" />
        </svg>
      )}
      {value}
    </span>
  );
}
