import type { ActivityEvent } from "~/api/types";
import { Badge, type BadgeProps } from "~/components/ui/badge";
import { type ActivityTone, activityTone } from "~/helpers/activity";
import { formatDateTime } from "~/helpers/deployments";

const TONE_VARIANTS: Record<ActivityTone, BadgeProps["variant"]> = {
  neutral: "secondary",
  success: "success",
  danger: "error",
  info: "info",
};

interface ActivityRowProps {
  readonly event: ActivityEvent;
  // The service and environment it is about, when the panel knows them.
  readonly entity: string | null;
}

// The summary quotes names the admin typed: React renders it as text, never as HTML.
export function ActivityRow({ event, entity }: ActivityRowProps) {
  return (
    <li className="grid grid-cols-1 items-center gap-2 px-4 py-2.5 text-sm sm:grid-cols-12 sm:gap-4">
      <time dateTime={event.at} className="text-muted-foreground sm:col-span-3">
        {formatDateTime(event.at)}
      </time>
      <span className="min-w-0 sm:col-span-3">
        <Badge variant={TONE_VARIANTS[activityTone(event.type)]}>{event.type}</Badge>
      </span>
      <span className="flex min-w-0 flex-col sm:col-span-4">
        <span className="truncate">{event.summary}</span>
        {entity && <span className="truncate text-muted-foreground text-xs">{entity}</span>}
      </span>
      <span className="truncate text-muted-foreground sm:col-span-2 sm:text-end">
        {event.actor}
      </span>
    </li>
  );
}
