import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { Link } from "@tanstack/react-router";
import { ArrowLeftIcon } from "lucide-react";
import { Suspense } from "react";

import { deploymentById } from "~/api/deployments";
import type { DeploymentId } from "~/api/types";
import { TAB_CLASSES } from "~/components/service-panel/panel-tabs";
import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { type DeploymentSearch, deploymentTitle, formatDateTime } from "~/helpers/deployments";

import { DeploymentLogs } from "./deployment-logs";
import { DeploymentStatusBadge } from "./deployment-status-badge";
import { RevisionDetails } from "./revision-details";

const FROM = "/projects/$projectId/$env/services/$serviceId/deployments/$deploymentId";

interface DeploymentDetailsProps {
  readonly deploymentId: DeploymentId;
  readonly tab: DeploymentSearch["tab"];
}

// Reference: infra/screens/05a-deployment-details.png. Each tab is a URL.
export function DeploymentDetails({ deploymentId, tab }: DeploymentDetailsProps) {
  const { data: deployment } = useLiveSuspenseQuery({ query: deploymentById(deploymentId) });
  if (!deployment) {
    return null;
  }

  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-3 px-6 pt-6">
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Retour aux déploiements"
          render={<Link from={FROM} to=".." />}
        >
          <ArrowLeftIcon />
        </Button>
        <DeploymentStatusBadge status={deployment.status} />
        <span className="min-w-0 flex-1 truncate font-medium text-sm">
          {deploymentTitle(deployment)}
        </span>
        <span className="text-muted-foreground text-xs">
          {formatDateTime(deployment.createdAt)}
        </span>
      </div>
      <nav className="mx-6 mt-4 flex gap-6 border-b">
        <Link
          from={FROM}
          to="."
          search={{}}
          activeOptions={{ exact: true }}
          className={TAB_CLASSES}
        >
          Détails
        </Link>
        <Link
          from={FROM}
          to="."
          search={{ tab: "build" }}
          activeOptions={{ exact: true }}
          className={TAB_CLASSES}
        >
          Logs de build
        </Link>
        <Link
          from={FROM}
          to="."
          search={{ tab: "deploy" }}
          activeOptions={{ exact: true }}
          className={TAB_CLASSES}
        >
          Logs d'exécution
        </Link>
      </nav>
      <Suspense
        fallback={
          <div className="flex justify-center p-8">
            <Spinner />
          </div>
        }
      >
        {tab === undefined && (
          <div className="p-6">
            <RevisionDetails deployment={deployment} />
          </div>
        )}
        {tab !== undefined && <DeploymentLogs deploymentId={deployment.id} stream={tab} />}
      </Suspense>
    </div>
  );
}
