"use client";

import { useState, useCallback, useEffect } from "react";
import { RepoContext } from "@/hooks/use-repo";

const STORAGE_KEY = "greport-repo";

export function RepoProvider({ children }: { children: React.ReactNode }) {
  const [owner, setOwner] = useState("");
  const [repo, setRepoName] = useState("");

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
  }, []);

  const setRepo = useCallback((o: string, r: string) => {
    setOwner(o);
    setRepoName(r);
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ owner: o, repo: r }));
  }, []);

  return (
    <RepoContext.Provider value={{ owner, repo, setRepo }}>
      {children}
    </RepoContext.Provider>
  );
}
