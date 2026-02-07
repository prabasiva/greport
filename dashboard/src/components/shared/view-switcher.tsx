"use client";

import { cn } from "@/lib/utils";

interface ViewOption {
  key: string;
  label: string;
}

interface ViewSwitcherProps {
  views: ViewOption[];
  activeView: string;
  onViewChange: (view: string) => void;
}

export function ViewSwitcher({ views, activeView, onViewChange }: ViewSwitcherProps) {
  return (
    <div className="inline-flex rounded-lg border border-gray-300 dark:border-gray-600">
      {views.map((view, i) => (
        <button
          key={view.key}
          onClick={() => onViewChange(view.key)}
          className={cn(
            "px-4 py-2 text-sm font-medium transition-colors",
            activeView === view.key
              ? "bg-blue-600 text-white"
              : "text-gray-600 hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-gray-800",
            i === 0 && "rounded-l-lg",
            i === views.length - 1 && "rounded-r-lg",
            i > 0 && "border-l border-gray-300 dark:border-gray-600",
          )}
        >
          {view.label}
        </button>
      ))}
    </div>
  );
}
