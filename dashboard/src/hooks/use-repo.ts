"use client";

import { createContext, useContext } from "react";

export type RepoMode = "single" | "aggregate";

export interface RepoContextType {
  owner: string;
  repo: string;
  mode: RepoMode;
  setRepo: (owner: string, repo: string) => void;
  setMode: (mode: RepoMode) => void;
}

export const RepoContext = createContext<RepoContextType>({
  owner: "",
  repo: "",
  mode: "single",
  setRepo: () => {},
  setMode: () => {},
});

export function useRepo() {
  return useContext(RepoContext);
}
