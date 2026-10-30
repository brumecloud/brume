import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";

import { allProjects } from "~/api/projects";

// Placeholder until the card dashboard (batch 1).
export function ProjectList() {
  const { data: projects } = useLiveSuspenseQuery({ query: allProjects });

  return (
    <ul className="flex flex-col gap-2">
      {projects.map((project) => (
        <li key={project.id}>
          <Link
            to="/projects/$projectId"
            params={{ projectId: project.id }}
            className="block rounded-lg border bg-card px-4 py-3 hover:bg-accent"
          >
            <span className="font-medium">{project.name}</span>
            <span className="block text-muted-foreground text-sm">{project.description}</span>
          </Link>
        </li>
      ))}
    </ul>
  );
}
