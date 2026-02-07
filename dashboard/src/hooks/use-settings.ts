"use client";

import { useState, useEffect, useCallback } from "react";

export type CalendarViewMode = "1" | "3";
export type PlanViewMode = "calendar" | "release-plan";

export interface AppSettings {
  calendarView: CalendarViewMode;
  planView: PlanViewMode;
}

const STORAGE_KEY = "greport-settings";

const defaultSettings: AppSettings = {
  calendarView: "3",
  planView: "calendar",
};

function loadSettings(): AppSettings {
  if (typeof window === "undefined") return defaultSettings;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultSettings;
    const parsed = JSON.parse(raw);
    return { ...defaultSettings, ...parsed };
  } catch {
    return defaultSettings;
  }
}

function saveSettings(settings: AppSettings) {
  if (typeof window === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export function useSettings() {
  const [settings, setSettingsState] = useState<AppSettings>(defaultSettings);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    setSettingsState(loadSettings());
    setLoaded(true);
  }, []);

  const updateSettings = useCallback((patch: Partial<AppSettings>) => {
    setSettingsState((prev) => {
      const next = { ...prev, ...patch };
      saveSettings(next);
      return next;
    });
  }, []);

  return { settings, updateSettings, loaded };
}
