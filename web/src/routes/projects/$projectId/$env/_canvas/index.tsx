import { createFileRoute } from "@tanstack/react-router";

// The canvas alone, without a service panel.
export const Route = createFileRoute("/projects/$projectId/$env/_canvas/")({});
