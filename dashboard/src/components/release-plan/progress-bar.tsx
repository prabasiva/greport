"use client";

import { cn } from "@/lib/utils";

interface ProgressBarProps {
  percent: number;
  className?: string;
}

export function ProgressBar({ percent, className }: ProgressBarProps) {
  const clamped = Math.min(100, Math.max(0, percent));
  const color =
    clamped >= 75
      ? "bg-green-500"
      : clamped >= 50
        ? "bg-blue-500"
        : clamped >= 25
          ? "bg-amber-500"
          : "bg-red-500";

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <div className="h-2 flex-1 overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
        <div
          className={cn("h-full rounded-full transition-all", color)}
          style={{ width: `${clamped}%` }}
        />
      </div>
      <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
        {Math.round(clamped)}%
      </span>
    </div>
  );
}
