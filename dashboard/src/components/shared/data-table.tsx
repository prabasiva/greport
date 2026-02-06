"use client";

import { useState } from "react";
import { cn } from "@/lib/utils";

export interface Column<T> {
  key: string;
  header: string;
  render: (item: T) => React.ReactNode;
  sortable?: boolean;
  className?: string;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  keyExtractor: (item: T) => string | number;
  onRowClick?: (item: T) => void;
  emptyMessage?: string;
}

export function DataTable<T>({
  columns,
  data,
  keyExtractor,
  onRowClick,
  emptyMessage = "No data available",
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  function handleSort(key: string) {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  const sortedData = sortKey
    ? [...data].sort((a, b) => {
        const col = columns.find((c) => c.key === sortKey);
        if (!col) return 0;
        const aVal = (a as Record<string, unknown>)[sortKey];
        const bVal = (b as Record<string, unknown>)[sortKey];
        const cmp =
          typeof aVal === "number" && typeof bVal === "number"
            ? aVal - bVal
            : String(aVal ?? "").localeCompare(String(bVal ?? ""));
        return sortDir === "asc" ? cmp : -cmp;
      })
    : data;

  if (data.length === 0) {
    return (
      <div className="py-12 text-center text-sm text-gray-500 dark:text-gray-400">
        {emptyMessage}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead className="bg-gray-50 dark:bg-gray-800">
          <tr>
            {columns.map((col) => (
              <th
                key={col.key}
                className={cn(
                  "px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400",
                  col.sortable && "cursor-pointer select-none hover:text-gray-700 dark:hover:text-gray-200",
                  col.className,
                )}
                onClick={() => col.sortable && handleSort(col.key)}
              >
                <div className="flex items-center gap-1">
                  {col.header}
                  {col.sortable && sortKey === col.key && (
                    <span>{sortDir === "asc" ? "\u2191" : "\u2193"}</span>
                  )}
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200 bg-white dark:divide-gray-700 dark:bg-gray-900">
          {sortedData.map((item) => (
            <tr
              key={keyExtractor(item)}
              className={cn(
                "transition-colors",
                onRowClick && "cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800",
              )}
              onClick={() => onRowClick?.(item)}
            >
              {columns.map((col) => (
                <td
                  key={col.key}
                  className={cn(
                    "whitespace-nowrap px-4 py-3 text-sm text-gray-900 dark:text-gray-100",
                    col.className,
                  )}
                >
                  {col.render(item)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
