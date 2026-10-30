import { useSuspenseQuery } from "@tanstack/react-query";
import { GitBranchIcon, GitCommitHorizontalIcon } from "lucide-react";
import type { ReactNode } from "react";

import { revisionQuery } from "~/api/deployments";
import type { Deployment } from "~/api/types";
import { Badge } from "~/components/ui/badge";

// Chips for what differs from the default only: a plain variable of the service has none.
const ORIGINS = { project: "Projet", environment: "Environnement" } as const;
const KINDS = { build: "Build", secret: "Secret" } as const;

// Reference: infra/screens/05a-deployment-details.png. A revision is immutable: what ran then.
export function RevisionDetails({ deployment }: { readonly deployment: Deployment }) {
  const { data: revision } = useSuspenseQuery(revisionQuery(deployment.id));

  return (
    <div className="flex flex-col gap-6">
      {deployment.commit && (
        <Block title="Source">
          <p className="whitespace-pre-wrap text-sm">{deployment.commit.message}</p>
          <p className="mt-2 flex gap-3 text-muted-foreground text-xs">
            <span className="flex items-center gap-1 font-mono">
              <GitCommitHorizontalIcon className="size-3.5" />
              {deployment.commit.sha}
            </span>
            <span className="flex items-center gap-1">
              <GitBranchIcon className="size-3.5" />
              {deployment.commit.branch}
            </span>
          </p>
        </Block>
      )}
      <Block title="Configuration">
        <dl className="grid grid-cols-1 gap-x-4 gap-y-2 text-sm sm:grid-cols-3">
          <dt className="text-muted-foreground">Build</dt>
          <dd className="sm:col-span-2">
            {revision.builder ?? "Aucun : image utilisée telle quelle"}
          </dd>
          <dt className="text-muted-foreground">Artefact</dt>
          <dd className="truncate font-mono sm:col-span-2">{revision.digest}</dd>
          <dt className="text-muted-foreground">Commande</dt>
          <dd className="font-mono sm:col-span-2">{revision.startCommand ?? "Celle de l'image"}</dd>
        </dl>
      </Block>
      <Block title={`Variables de la révision (${revision.variables.length})`}>
        <ul className="flex flex-col gap-2">
          {revision.variables.map((variable) => (
            <li key={variable.name} className="flex flex-wrap items-center gap-2 text-sm">
              <span className="font-mono">{variable.name}</span>
              {variable.kind !== "plain" && <Badge variant="outline">{KINDS[variable.kind]}</Badge>}
              {variable.origin !== "service" && (
                <Badge variant="secondary">{ORIGINS[variable.origin]}</Badge>
              )}
              <span className="ms-auto truncate font-mono text-muted-foreground">
                {variable.value ?? "••••••••"}
              </span>
            </li>
          ))}
        </ul>
      </Block>
    </div>
  );
}

function Block({ title, children }: { readonly title: string; readonly children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-medium text-sm">{title}</h3>
      <div className="rounded-xl border bg-background p-4">{children}</div>
    </section>
  );
}
