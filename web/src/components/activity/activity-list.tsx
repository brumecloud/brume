import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useSuspenseInfiniteQuery } from "@tanstack/react-query";
import { HistoryIcon } from "lucide-react";
import { useLayoutEffect, useRef } from "react";

import { activityQuery } from "~/api/activity";
import { allEnvironments } from "~/api/projects";
import { allServices } from "~/api/services";
import type { ProjectId } from "~/api/types";
import { Button } from "~/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { activityEntity, activityRows } from "~/helpers/activity";

import { ActivityRow } from "./activity-row";

// The events that made the project what it is, in the order the engine recorded them: the newest
// at the bottom, where the view opens, older pages loaded above.
export function ActivityList({ projectId }: { readonly projectId: ProjectId }) {
  const activity = useSuspenseInfiniteQuery(activityQuery(projectId));
  const { data: environments } = useLiveSuspenseQuery({ query: allEnvironments });
  const { data: services } = useLiveSuspenseQuery({ query: allServices });
  const events = activityRows(activity.data.pages);
  const list = useRef<HTMLOListElement>(null);

  // oxlint-disable-next-line brume/no-unreasoned-effect -- scrolls the DOM to the newest event once, before the first paint; earlier pages loaded later keep the scroll where it is
  useLayoutEffect(() => {
    list.current?.lastElementChild?.scrollIntoView({ block: "end" });
  }, []);

  return (
    <div className="mx-auto flex w-full max-w-4xl flex-col gap-4 p-6">
      <div>
        <h2 className="font-medium">Activity</h2>
        <p className="text-muted-foreground text-sm">
          Every event of the project, in the order the engine recorded it.
        </p>
      </div>
      {activity.hasNextPage && (
        <Button
          variant="outline"
          size="sm"
          className="self-center"
          loading={activity.isFetchingNextPage}
          onClick={() => void activity.fetchNextPage()}
        >
          Load earlier events
        </Button>
      )}
      {events.length > 0 ? (
        <ol
          ref={list}
          className="flex flex-col divide-y overflow-hidden rounded-xl border bg-background"
        >
          {events.map((event) => (
            <ActivityRow
              key={event.seq}
              event={event}
              entity={activityEntity(event, environments, services)}
            />
          ))}
        </ol>
      ) : (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <HistoryIcon />
            </EmptyMedia>
            <EmptyTitle>No activity</EmptyTitle>
            <EmptyDescription>Nothing happened in this project yet.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      )}
    </div>
  );
}
