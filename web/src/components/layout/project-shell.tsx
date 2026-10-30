import type { ReactNode } from "react";

import type { ProjectId } from "~/api/types";
import { EnvironmentSwitcher } from "~/components/environments/environment-switcher";
import { ProjectSwitcher } from "~/components/projects/project-switcher";

import { BrumeLogo } from "./brume-logo";
import { ProjectRail } from "./project-rail";

interface ProjectShellProps {
  readonly projectId: ProjectId;
  readonly environmentName: string;
  readonly children: ReactNode;
}

// Reference: infra/screens/03a-project-canvas-prod.png and 03c.
export function ProjectShell({ projectId, environmentName, children }: ProjectShellProps) {
  return (
    <div className="flex h-dvh flex-col bg-sidebar">
      <header className="flex h-14 shrink-0 items-center gap-2 px-4">
        <BrumeLogo />
        <span className="mx-2 h-5 w-px bg-border" />
        <ProjectSwitcher projectId={projectId} />
        <span className="text-muted-foreground">/</span>
        <EnvironmentSwitcher projectId={projectId} environmentName={environmentName} />
      </header>
      <div className="flex min-h-0 flex-1">
        <ProjectRail projectId={projectId} environmentName={environmentName} />
        <main className="m-2 min-w-0 flex-1 overflow-auto rounded-xl border bg-background md:ml-0">
          {children}
        </main>
      </div>
    </div>
  );
}
