"use client";

import { cn } from "@/lib/utils";
import type { PaginationMeta } from "@/types/api";

interface PaginationProps {
  meta: PaginationMeta;
  onPageChange: (page: number) => void;
}

export function Pagination({ meta, onPageChange }: PaginationProps) {
  const { page, total_pages } = meta;

  if (total_pages <= 1) return null;

  const pages: (number | "...")[] = [];
  if (total_pages <= 7) {
    for (let i = 1; i <= total_pages; i++) pages.push(i);
  } else {
    pages.push(1);
    if (page > 3) pages.push("...");
    for (let i = Math.max(2, page - 1); i <= Math.min(total_pages - 1, page + 1); i++) {
      pages.push(i);
    }
    if (page < total_pages - 2) pages.push("...");
    pages.push(total_pages);
  }

  return (
    <nav className="flex items-center justify-between border-t border-gray-200 px-4 py-3 dark:border-gray-700 sm:px-6">
      <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
        <p className="text-sm text-gray-700 dark:text-gray-300">
          Page <span className="font-medium">{page}</span> of{" "}
          <span className="font-medium">{total_pages}</span>{" "}
          ({meta.total} total items)
        </p>
        <div className="flex gap-1">
          <button
            onClick={() => onPageChange(page - 1)}
            disabled={page <= 1}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed dark:text-gray-300 dark:hover:bg-gray-800"
          >
            Previous
          </button>
          {pages.map((p, i) =>
            p === "..." ? (
              <span key={`dots-${i}`} className="px-2 py-1.5 text-sm text-gray-500">
                ...
              </span>
            ) : (
              <button
                key={p}
                onClick={() => onPageChange(p)}
                className={cn(
                  "rounded-md px-3 py-1.5 text-sm font-medium",
                  p === page
                    ? "bg-blue-600 text-white"
                    : "text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800",
                )}
              >
                {p}
              </button>
            ),
          )}
          <button
            onClick={() => onPageChange(page + 1)}
            disabled={page >= total_pages}
            className="rounded-md px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed dark:text-gray-300 dark:hover:bg-gray-800"
          >
            Next
          </button>
        </div>
      </div>
    </nav>
  );
}
