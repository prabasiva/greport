"use client";

import { useState, useMemo } from "react";
import { useRepo } from "@/hooks/use-repo";
import { useCalendar, useAggregateCalendar } from "@/hooks/use-api";
import { useSettings, type CalendarViewMode } from "@/hooks/use-settings";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { CalendarNav } from "@/components/calendar/calendar-nav";
import { CalendarFilters, type FilterState } from "@/components/calendar/calendar-filters";
import { CalendarGrid } from "@/components/calendar/calendar-grid";
import { EventPopover } from "@/components/calendar/event-popover";
import type { CalendarEvent } from "@/types/api";

function formatDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export default function CalendarPage() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregateCalendarView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;
  return <SingleCalendarView owner={owner} repo={repo} />;
}

function SingleCalendarView({ owner, repo }: { owner: string; repo: string }) {
  const { settings, updateSettings } = useSettings();
  const [baseMonth, setBaseMonth] = useState(() => new Date());
  const [filters, setFilters] = useState<FilterState>({
    issues: true,
    milestones: true,
    releases: true,
    pulls: true,
  });
  const [selectedEvents, setSelectedEvents] = useState<CalendarEvent[] | null>(null);

  const viewMode = settings.calendarView;
  const { startDate, endDate, typesParam } = useDateRange(baseMonth, filters, viewMode);

  const { data, error, isLoading } = useCalendar(owner, repo, {
    start_date: startDate,
    end_date: endDate,
    types: typesParam,
  });

  const events = data?.data?.events || [];

  const handleViewModeChange = (mode: CalendarViewMode) => {
    updateSettings({ calendarView: mode });
  };

  return (
    <CalendarLayout
      title="Calendar"
      subtitle={`${owner}/${repo}`}
      baseMonth={baseMonth}
      setBaseMonth={setBaseMonth}
      viewMode={viewMode}
      onViewModeChange={handleViewModeChange}
      filters={filters}
      setFilters={setFilters}
      events={events}
      isLoading={isLoading}
      error={error}
      selectedEvents={selectedEvents}
      setSelectedEvents={setSelectedEvents}
      summary={data?.data?.summary}
    />
  );
}

function AggregateCalendarView() {
  const { settings, updateSettings } = useSettings();
  const [baseMonth, setBaseMonth] = useState(() => new Date());
  const [filters, setFilters] = useState<FilterState>({
    issues: true,
    milestones: true,
    releases: true,
    pulls: true,
  });
  const [selectedEvents, setSelectedEvents] = useState<CalendarEvent[] | null>(null);

  const viewMode = settings.calendarView;
  const { startDate, endDate, typesParam } = useDateRange(baseMonth, filters, viewMode);

  const { data, error, isLoading } = useAggregateCalendar({
    start_date: startDate,
    end_date: endDate,
    types: typesParam,
  });

  const events = data?.data?.events || [];

  const handleViewModeChange = (mode: CalendarViewMode) => {
    updateSettings({ calendarView: mode });
  };

  return (
    <CalendarLayout
      title="Calendar"
      subtitle="All repositories"
      baseMonth={baseMonth}
      setBaseMonth={setBaseMonth}
      viewMode={viewMode}
      onViewModeChange={handleViewModeChange}
      filters={filters}
      setFilters={setFilters}
      events={events}
      isLoading={isLoading}
      error={error}
      selectedEvents={selectedEvents}
      setSelectedEvents={setSelectedEvents}
      summary={data?.data?.summary}
    />
  );
}

function useDateRange(baseMonth: Date, filters: FilterState, viewMode: CalendarViewMode) {
  return useMemo(() => {
    let start: Date;
    let end: Date;

    if (viewMode === "1") {
      start = new Date(baseMonth.getFullYear(), baseMonth.getMonth(), 1);
      end = new Date(baseMonth.getFullYear(), baseMonth.getMonth() + 1, 0);
    } else {
      start = new Date(baseMonth.getFullYear(), baseMonth.getMonth() - 1, 1);
      end = new Date(baseMonth.getFullYear(), baseMonth.getMonth() + 2, 0);
    }

    const activeTypes: string[] = [];
    if (filters.issues) activeTypes.push("issues");
    if (filters.milestones) activeTypes.push("milestones");
    if (filters.releases) activeTypes.push("releases");
    if (filters.pulls) activeTypes.push("pulls");

    return {
      startDate: formatDate(start),
      endDate: formatDate(end),
      typesParam: activeTypes.join(","),
    };
  }, [baseMonth, filters, viewMode]);
}

interface CalendarLayoutProps {
  title: string;
  subtitle: string;
  baseMonth: Date;
  setBaseMonth: (d: Date) => void;
  viewMode: CalendarViewMode;
  onViewModeChange: (mode: CalendarViewMode) => void;
  filters: FilterState;
  setFilters: (f: FilterState) => void;
  events: CalendarEvent[];
  isLoading: boolean;
  error: Error | undefined;
  selectedEvents: CalendarEvent[] | null;
  setSelectedEvents: (e: CalendarEvent[] | null) => void;
  summary?: { total_events: number; by_type: Record<string, number> };
}

function CalendarLayout({
  title,
  subtitle,
  baseMonth,
  setBaseMonth,
  viewMode,
  onViewModeChange,
  filters,
  setFilters,
  events,
  isLoading,
  error,
  selectedEvents,
  setSelectedEvents,
  summary,
}: CalendarLayoutProps) {
  const months = useMemo(() => {
    const result: { year: number; month: number }[] = [];
    if (viewMode === "1") {
      result.push({ year: baseMonth.getFullYear(), month: baseMonth.getMonth() });
    } else {
      for (let i = -1; i <= 1; i++) {
        const d = new Date(baseMonth.getFullYear(), baseMonth.getMonth() + i, 1);
        result.push({ year: d.getFullYear(), month: d.getMonth() });
      }
    }
    return result;
  }, [baseMonth, viewMode]);

  const handlePrev = () =>
    setBaseMonth(new Date(baseMonth.getFullYear(), baseMonth.getMonth() - 1, 1));
  const handleNext = () =>
    setBaseMonth(new Date(baseMonth.getFullYear(), baseMonth.getMonth() + 1, 1));
  const handleToday = () => setBaseMonth(new Date());

  const handleToggle = (key: keyof FilterState) => {
    setFilters({ ...filters, [key]: !filters[key] });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">{title}</h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
      </div>

      <CalendarNav
        baseMonth={baseMonth}
        viewMode={viewMode}
        onPrev={handlePrev}
        onNext={handleNext}
        onToday={handleToday}
        onViewModeChange={onViewModeChange}
      />

      <div className="flex items-center justify-between">
        <CalendarFilters filters={filters} onToggle={handleToggle} />
        {summary && (
          <span className="text-sm text-gray-500 dark:text-gray-400">
            {summary.total_events} event{summary.total_events !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {isLoading && <PageLoading />}
      {error && <ErrorDisplay message={error.message} />}

      {!isLoading && !error && (
        <div
          className={
            viewMode === "1"
              ? "grid gap-6 grid-cols-1"
              : "grid gap-6 lg:grid-cols-3"
          }
        >
          {months.map(({ year, month }) => (
            <CalendarGrid
              key={`${year}-${month}`}
              year={year}
              month={month}
              events={events}
              onSelectDay={setSelectedEvents}
              expanded={viewMode === "1"}
            />
          ))}
        </div>
      )}

      {selectedEvents && (
        <EventPopover
          events={selectedEvents}
          onClose={() => setSelectedEvents(null)}
        />
      )}
    </div>
  );
}
