"use client";

import { useState, useCallback, useEffect } from "react";
import { RepoContext, type RepoMode } from "@/hooks/use-repo";

const STORAGE_KEY = "greport-repo";
const MODE_STORAGE_KEY = "greport-mode";

export function RepoProvider({ children }: { children: React.ReactNode }) {
  const [owner, setOwner] = useState("");
  const [repo, setRepoName] = useState("");
  const [mode, setModeState] = useState<RepoMode>("aggregate");

  useEffect(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        const { owner: o, repo: r } = JSON.parse(saved);
        if (o && r) {
          setOwner(o);
          setRepoName(r);
        }
      } catch {
        // ignore invalid JSON
      }
    }
    const savedMode = localStorage.getItem(MODE_STORAGE_KEY);
    if (savedMode === "single" || savedMode === "aggregate") {
      setModeState(savedMode);
    }
  }, []);

  const setRepo = useCallback((o: string, r: string) => {
    setOwner(o);
    setRepoName(r);
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ owner: o, repo: r }));
  }, []);

  const setMode = useCallback((m: RepoMode) => {
    setModeState(m);
    localStorage.setItem(MODE_STORAGE_KEY, m);
  }, []);

  return (
    <RepoContext.Provider value={{ owner, repo, mode, setRepo, setMode }}>
      {children}
    </RepoContext.Provider>
  );
}
