import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach } from "vitest";
import { useSettings } from "../use-settings";

describe("useSettings", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("returns default settings initially", () => {
    const { result } = renderHook(() => useSettings());
    expect(result.current.settings.calendarView).toBe("3");
    expect(result.current.settings.planView).toBe("calendar");
  });

  it("updates calendarView", () => {
    const { result } = renderHook(() => useSettings());
    act(() => {
      result.current.updateSettings({ calendarView: "1" });
    });
    expect(result.current.settings.calendarView).toBe("1");
  });

  it("updates planView", () => {
    const { result } = renderHook(() => useSettings());
    act(() => {
      result.current.updateSettings({ planView: "release-plan" });
    });
    expect(result.current.settings.planView).toBe("release-plan");
  });

  it("persists settings to localStorage", () => {
    const { result } = renderHook(() => useSettings());
    act(() => {
      result.current.updateSettings({ calendarView: "1" });
    });
    const stored = JSON.parse(localStorage.getItem("greport-settings") || "{}");
    expect(stored.calendarView).toBe("1");
  });

  it("loads persisted settings on mount", () => {
    localStorage.setItem(
      "greport-settings",
      JSON.stringify({ calendarView: "1", planView: "release-plan" }),
    );
    const { result } = renderHook(() => useSettings());
    // After useEffect runs, settings should be loaded
    expect(result.current.settings.calendarView).toBe("1");
  });

  it("merges partial updates without losing other settings", () => {
    const { result } = renderHook(() => useSettings());
    act(() => {
      result.current.updateSettings({ calendarView: "1" });
    });
    act(() => {
      result.current.updateSettings({ planView: "release-plan" });
    });
    expect(result.current.settings.calendarView).toBe("1");
    expect(result.current.settings.planView).toBe("release-plan");
  });

  it("falls back to defaults on corrupt localStorage data", () => {
    localStorage.setItem("greport-settings", "not-json");
    const { result } = renderHook(() => useSettings());
    expect(result.current.settings.calendarView).toBe("3");
    expect(result.current.settings.planView).toBe("calendar");
  });
});
