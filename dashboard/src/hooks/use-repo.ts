"use client";

import { createContext, useContext } from "react";

export interface RepoContextType {
  owner: string;
  repo: string;
  setRepo: (owner: string, repo: string) => void;
}

export const RepoContext = createContext<RepoContextType>({
  owner: "",
  repo: "",
  setRepo: () => {},
});

export function useRepo() {
  return useContext(RepoContext);
}
