import { Link } from "@tanstack/react-router";
import { CloudOffIcon, TriangleAlertIcon } from "lucide-react";

import { Button } from "~/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";

export function NotFoundPage() {
  return (
    <Empty className="h-dvh">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <CloudOffIcon />
        </EmptyMedia>
        <EmptyTitle>Page not found</EmptyTitle>
        <EmptyDescription>
          This project, environment or service doesn't exist, or no longer does.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button variant="outline" render={<Link to="/projects" />}>
          Back to projects
        </Button>
      </EmptyContent>
    </Empty>
  );
}

// An error message can come from a crafted URL: never display it.
export function ErrorPage() {
  return (
    <Empty className="h-dvh">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <TriangleAlertIcon />
        </EmptyMedia>
        <EmptyTitle>Something went wrong</EmptyTitle>
        <EmptyDescription>
          Reload the page. If the error comes back, check the Brume logs.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button variant="outline" render={<Link to="/projects" />}>
          Back to projects
        </Button>
      </EmptyContent>
    </Empty>
  );
}
