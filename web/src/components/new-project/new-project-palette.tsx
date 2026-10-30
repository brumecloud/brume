import { Suspense } from "react";

import { Spinner } from "~/components/ui/spinner";
import type { NewProjectStep } from "~/helpers/new-project";

import { DatabaseStep } from "./database-step";
import { ImageStep } from "./image-step";
import { RepositoryStep } from "./repository-step";
import { SourceStep } from "./source-step";
import { StepHeader } from "./step-header";
import { useCreateProject } from "./use-create-project";

export const STEP_TITLES: Record<NewProjectStep, string> = {
  github: "Dépôt GitHub",
  static: "Site statique",
  database: "Base de données",
  image: "Image Docker",
};

export function NewProjectPalette({ step }: { readonly step: NewProjectStep | undefined }) {
  const create = useCreateProject();
  const stepProps = { disabled: create.isPending, onCreate: create.mutate };

  return (
    <div className="flex h-fit w-full max-w-md flex-col overflow-hidden rounded-2xl border bg-popover shadow-lg">
      {step && <StepHeader title={STEP_TITLES[step]} />}
      <Suspense
        fallback={
          <div className="flex justify-center p-8">
            <Spinner />
          </div>
        }
      >
        {!step && <SourceStep {...stepProps} />}
        {(step === "github" || step === "static") && <RepositoryStep kind={step} {...stepProps} />}
        {step === "database" && <DatabaseStep {...stepProps} />}
        {step === "image" && <ImageStep {...stepProps} />}
      </Suspense>
      {create.error && (
        <p role="alert" className="border-t px-4 py-3 text-destructive-foreground text-sm">
          {create.error.message}
        </p>
      )}
    </div>
  );
}
