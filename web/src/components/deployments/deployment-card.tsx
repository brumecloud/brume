import { Link } from "@tanstack/react-router";
import { EllipsisVerticalIcon, GitBranchIcon, GitCommitHorizontalIcon } from "lucide-react";
import { useState } from "react";

import type { Deployment, Service } from "~/api/types";
import { Button } from "~/components/ui/button";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "~/components/ui/menu";
import { deploymentTitle, formatDateTime } from "~/helpers/deployments";

import { DeploymentStatusBadge } from "./deployment-status-badge";
import { RollbackDialog } from "./rollback-dialog";

interface DeploymentCardProps {
  readonly service: Service;
  readonly deployment: Deployment;
}

export function DeploymentCard({ service, deployment }: DeploymentCardProps) {
  const [rollbackOpen, setRollbackOpen] = useState(false);

  return (
    <article className="flex items-center gap-4 rounded-xl border bg-background px-4 py-3">
      <DeploymentStatusBadge status={deployment.status} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <Link
          from="/projects/$projectId/$env/services/$serviceId/deployments"
          to="./$deploymentId"
          params={{ deploymentId: deployment.id }}
          className="truncate text-sm hover:underline"
        >
          {deploymentTitle(deployment)}
        </Link>
        <span className="flex flex-wrap items-center gap-x-3 gap-y-1 text-muted-foreground text-xs">
          <span>
            {formatDateTime(deployment.createdAt)} · par {deployment.author}
          </span>
          {deployment.commit && (
            <>
              <span className="flex items-center gap-1 font-mono">
                <GitCommitHorizontalIcon className="size-3.5" />
                {deployment.commit.sha.slice(0, 7)}
              </span>
              <span className="flex items-center gap-1">
                <GitBranchIcon className="size-3.5" />
                {deployment.commit.branch}
              </span>
            </>
          )}
        </span>
      </div>
      {deployment.status !== "active" && (
        <Menu>
          <MenuTrigger
            render={<Button variant="ghost" size="icon-sm" aria-label="Actions du déploiement" />}
          >
            <EllipsisVerticalIcon />
          </MenuTrigger>
          <MenuPopup align="end">
            <MenuItem disabled={!deployment.rollbackable} onClick={() => setRollbackOpen(true)}>
              {deployment.rollbackable
                ? "Revenir à ce déploiement"
                : "Artefact purgé : retour impossible"}
            </MenuItem>
          </MenuPopup>
        </Menu>
      )}
      <RollbackDialog
        service={service}
        deployment={deployment}
        open={rollbackOpen}
        onOpenChange={setRollbackOpen}
      />
    </article>
  );
}
