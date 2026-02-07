"use client";

import useSWR from "swr";
import { useRepo } from "@/hooks/use-repo";
import { reposUrl } from "@/lib/api";
import type { ApiResponse, RepoSummary } from "@/types/api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:9423";

async function directFetcher<T>(url: string): Promise<T> {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error(body?.error?.message || res.statusText);
  }
  return res.json();
}

export interface RepoEntry {
  owner: string;
  name: string;
  fullName: string;
}

export function useRepos() {
  const { owner, repo } = useRepo();

  const { data } = useSWR<ApiResponse<RepoSummary[]>>(
    `${API_BASE}${reposUrl()}`,
    directFetcher,
    { revalidateOnFocus: false, dedupingInterval: 60000 },
  );

  const repos: RepoEntry[] = (data?.data || []).map((r) => ({
    owner: r.owner,
    name: r.name,
    fullName: r.full_name,
  }));

  const activeRepo =
    owner && repo ? { owner, name: repo } : null;

  return { activeRepo, repos };
}
