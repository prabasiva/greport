export function LoadingSpinner({ className }: { className?: string }) {
  return (
    <div className={className}>
      <div className="flex items-center justify-center py-12">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-200 border-t-blue-600" />
      </div>
    </div>
  );
}

export function LoadingSkeleton({ lines = 3 }: { lines?: number }) {
  return (
    <div className="animate-pulse space-y-3">
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="h-4 rounded bg-gray-200 dark:bg-gray-700"
          style={{ width: `${85 - i * 15}%` }}
        />
      ))}
    </div>
  );
}

export function PageLoading() {
  return (
    <div className="flex min-h-[400px] items-center justify-center">
      <div className="text-center">
        <div className="mx-auto h-12 w-12 animate-spin rounded-full border-4 border-gray-200 border-t-blue-600" />
        <p className="mt-4 text-sm text-gray-500 dark:text-gray-400">
          Loading data...
        </p>
      </div>
    </div>
  );
}
