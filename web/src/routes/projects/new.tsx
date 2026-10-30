import { createFileRoute } from "@tanstack/react-router";

import { NewProjectHeader } from "~/components/new-project/new-project-header";
import { NewProjectPalette } from "~/components/new-project/new-project-palette";
import { parseNewProjectSearch } from "~/helpers/new-project";

export const Route = createFileRoute("/projects/new")({
  validateSearch: parseNewProjectSearch,
  component: NewProjectPage,
});

function NewProjectPage() {
  const { step } = Route.useSearch();

  return (
    <div className="flex h-dvh flex-col bg-sidebar">
      <NewProjectHeader step={step} />
      <main className="m-2 mt-0 flex min-h-0 flex-1 justify-center overflow-auto rounded-xl border bg-background bg-dots px-4 pt-32">
        <NewProjectPalette step={step} />
      </main>
    </div>
  );
}
