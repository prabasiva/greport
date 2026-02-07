"use client";

import { Suspense, useState, useMemo } from "react";
import { useSearchParams } from "next/navigation";
import { useRepo } from "@/hooks/use-repo";
import {
  useCalendar,
  useAggregateCalendar,
  useReleasePlan,
  useAggregateReleasePlan,
} from "@/hooks/use-api";
import { useSettings, type CalendarViewMode, type PlanViewMode } from "@/hooks/use-settings";
import { PageLoading } from "@/components/shared/loading";
import { ErrorDisplay, NoRepoSelected } from "@/components/shared/error-display";
import { ViewSwitcher } from "@/components/shared/view-switcher";
import { CalendarNav } from "@/components/calendar/calendar-nav";
import { CalendarFilters, type FilterState } from "@/components/calendar/calendar-filters";
import { CalendarGrid } from "@/components/calendar/calendar-grid";
import { EventPopover } from "@/components/calendar/event-popover";
import { ReleasePlanView } from "@/components/release-plan/release-plan-view";
import type { CalendarEvent } from "@/types/api";

const VIEW_OPTIONS = [
  { key: "calendar", label: "Calendar View" },
  { key: "release-plan", label: "Release Plan" },
];

function formatDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export default function PlanningPage() {
  return (
    <Suspense fallback={<PageLoading />}>
      <PlanningContent />
    </Suspense>
  );
}

function PlanningContent() {
  const { owner, repo, mode } = useRepo();

  if (mode === "aggregate") {
    return <AggregatePlanningView />;
  }

  if (!owner || !repo) return <NoRepoSelected />;
  return <SinglePlanningView owner={owner} repo={repo} />;
}

function SinglePlanningView({ owner, repo }: { owner: string; repo: string }) {
  const { settings, updateSettings } = useSettings();
  const searchParams = useSearchParams();
  const urlView = searchParams.get("view") as PlanViewMode | null;
  const activeView = urlView || settings.planView;

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

  const calendarResult = useCalendar(
    activeView === "calendar" ? owner : "",
    activeView === "calendar" ? repo : "",
    { start_date: startDate, end_date: endDate, types: typesParam },
  );

  const releasePlanResult = useReleasePlan(
    activeView === "release-plan" ? owner : "",
    activeView === "release-plan" ? repo : "",
  );

  const handleViewChange = (view: string) => {
    updateSettings({ planView: view as PlanViewMode });
  };

  const handleViewModeChange = (mode: CalendarViewMode) => {
    updateSettings({ calendarView: mode });
  };

  return (
    <PlanningLayout
      title="Planning"
      subtitle={`${owner}/${repo}`}
      activeView={activeView}
      onViewChange={handleViewChange}
      baseMonth={baseMonth}
      setBaseMonth={setBaseMonth}
      viewMode={viewMode}
      onViewModeChange={handleViewModeChange}
      filters={filters}
      setFilters={setFilters}
      calendarEvents={calendarResult.data?.data?.events || []}
      calendarSummary={calendarResult.data?.data?.summary}
      releasePlan={releasePlanResult.data?.data}
      isLoading={activeView === "calendar" ? calendarResult.isLoading : releasePlanResult.isLoading}
      error={activeView === "calendar" ? calendarResult.error : releasePlanResult.error}
      selectedEvents={selectedEvents}
      setSelectedEvents={setSelectedEvents}
    />
  );
}

function AggregatePlanningView() {
  const { settings, updateSettings } = useSettings();
  const searchParams = useSearchParams();
  const urlView = searchParams.get("view") as PlanViewMode | null;
  const activeView = urlView || settings.planView;

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

  const calendarResult = useAggregateCalendar(
    activeView === "calendar"
      ? { start_date: startDate, end_date: endDate, types: typesParam }
      : undefined,
  );

  const releasePlanResult = useAggregateReleasePlan(
    activeView === "release-plan" ? {} : undefined,
  );

  const handleViewChange = (view: string) => {
    updateSettings({ planView: view as PlanViewMode });
  };

  const handleViewModeChange = (mode: CalendarViewMode) => {
    updateSettings({ calendarView: mode });
  };

  return (
    <PlanningLayout
      title="Planning"
      subtitle="All repositories"
      activeView={activeView}
      onViewChange={handleViewChange}
      baseMonth={baseMonth}
      setBaseMonth={setBaseMonth}
      viewMode={viewMode}
      onViewModeChange={handleViewModeChange}
      filters={filters}
      setFilters={setFilters}
      calendarEvents={calendarResult.data?.data?.events || []}
      calendarSummary={calendarResult.data?.data?.summary}
      releasePlan={releasePlanResult.data?.data}
      isLoading={activeView === "calendar" ? calendarResult.isLoading : releasePlanResult.isLoading}
      error={activeView === "calendar" ? calendarResult.error : releasePlanResult.error}
      selectedEvents={selectedEvents}
      setSelectedEvents={setSelectedEvents}
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

interface PlanningLayoutProps {
  title: string;
  subtitle: string;
  activeView: string;
  onViewChange: (view: string) => void;
  baseMonth: Date;
  setBaseMonth: (d: Date) => void;
  viewMode: CalendarViewMode;
  onViewModeChange: (mode: CalendarViewMode) => void;
  filters: FilterState;
  setFilters: (f: FilterState) => void;
  calendarEvents: CalendarEvent[];
  calendarSummary?: { total_events: number; by_type: Record<string, number> };
  releasePlan?: import("@/types/api").ReleasePlan;
  isLoading: boolean;
  error: Error | undefined;
  selectedEvents: CalendarEvent[] | null;
  setSelectedEvents: (e: CalendarEvent[] | null) => void;
}

function PlanningLayout({
  title,
  subtitle,
  activeView,
  onViewChange,
  baseMonth,
  setBaseMonth,
  viewMode,
  onViewModeChange,
  filters,
  setFilters,
  calendarEvents,
  calendarSummary,
  releasePlan,
  isLoading,
  error,
  selectedEvents,
  setSelectedEvents,
}: PlanningLayoutProps) {
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
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">{title}</h2>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
        </div>
        <ViewSwitcher
          views={VIEW_OPTIONS}
          activeView={activeView}
          onViewChange={onViewChange}
        />
      </div>

      {activeView === "calendar" && (
        <>
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
            {calendarSummary && (
              <span className="text-sm text-gray-500 dark:text-gray-400">
                {calendarSummary.total_events} event{calendarSummary.total_events !== 1 ? "s" : ""}
              </span>
            )}
          </div>
        </>
      )}

      {isLoading && <PageLoading />}
      {error && <ErrorDisplay message={error.message} />}

      {!isLoading && !error && activeView === "calendar" && (
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
              events={calendarEvents}
              onSelectDay={setSelectedEvents}
              expanded={viewMode === "1"}
            />
          ))}
        </div>
      )}

      {!isLoading && !error && activeView === "release-plan" && releasePlan && (
        <ReleasePlanView data={releasePlan} />
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
