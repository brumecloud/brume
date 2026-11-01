import { infiniteQueryOptions } from "@tanstack/react-query";

import { parseActivityPage } from "~/helpers/activity";

import { apiGet } from "./client";
import type { ActivityPage, ProjectId } from "./types";

const PAGE_SIZE = 100;

function activityPage(projectId: ProjectId, before: number | null): Promise<ActivityPage> {
  const query = new URLSearchParams({ projectId, limit: String(PAGE_SIZE) });
  if (before !== null) {
    query.set("before", String(before));
  }
  return apiGet(`/api/events?${query}`, parseActivityPage);
}

// Newest page first; each next page is older. Fresh on every visit: the log only grows.
export const activityQuery = (projectId: ProjectId) =>
  infiniteQueryOptions({
    queryKey: ["activity", projectId],
    queryFn: ({ pageParam }) => activityPage(projectId, pageParam),
    initialPageParam: null as number | null,
    getNextPageParam: (page) => page.nextBefore,
    staleTime: 0,
  });
