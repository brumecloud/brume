import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/projects/$projectId/$env/_canvas/services/$serviceId/")({
  beforeLoad: ({ params }) => {
    throw redirect({ to: "/projects/$projectId/$env/services/$serviceId/deployments", params });
  },
});
