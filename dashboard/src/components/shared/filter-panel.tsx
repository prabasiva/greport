"use client";

import { cn } from "@/lib/utils";

interface FilterOption {
  label: string;
  value: string;
}

interface FilterPanelProps {
  filters: {
    key: string;
    label: string;
    options: FilterOption[];
    value: string;
    onChange: (value: string) => void;
  }[];
  onClear?: () => void;
}

export function FilterPanel({ filters, onClear }: FilterPanelProps) {
  const hasActiveFilters = filters.some((f) => f.value !== "" && f.value !== "all");

  return (
    <div className="flex flex-wrap items-center gap-3 rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-700 dark:bg-gray-800/50">
      {filters.map((filter) => (
        <div key={filter.key} className="flex items-center gap-2">
          <label
            htmlFor={filter.key}
            className="text-xs font-medium text-gray-500 dark:text-gray-400"
          >
            {filter.label}
          </label>
          <select
            id={filter.key}
            value={filter.value}
            onChange={(e) => filter.onChange(e.target.value)}
            className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          >
            {filter.options.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
      ))}
      {hasActiveFilters && onClear && (
        <button
          onClick={onClear}
          className="ml-auto text-xs font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400"
        >
          Clear filters
        </button>
      )}
    </div>
  );
}

interface ExportButtonProps {
  onExportCsv?: () => void;
  onExportJson?: () => void;
  className?: string;
}

export function ExportButton({ onExportCsv, onExportJson, className }: ExportButtonProps) {
  return (
    <div className={cn("flex gap-2", className)}>
      {onExportCsv && (
        <button
          onClick={onExportCsv}
          className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
        >
          Export CSV
        </button>
      )}
      {onExportJson && (
        <button
          onClick={onExportJson}
          className="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
        >
          Export JSON
        </button>
      )}
    </div>
  );
}
