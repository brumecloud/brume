import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { GlobeIcon, RotateCwIcon } from "lucide-react";

import { redeploy, serviceDeployments } from "~/api/deployments";
import { serviceById } from "~/api/services";
import type { ServiceId } from "~/api/types";
import { Button } from "~/components/ui/button";
import { splitDeployments } from "~/helpers/deployments";
import { domainUrl } from "~/helpers/services";

import { DeploymentCard } from "./deployment-card";

// Reference: infra/screens/04a-service-deployments.png.
export function DeploymentList({ serviceId }: { readonly serviceId: ServiceId }) {
  const { data: service } = useLiveSuspenseQuery({ query: serviceById(serviceId) });
  const { data: deployments } = useLiveSuspenseQuery({ query: serviceDeployments(serviceId) });
  const { active, others } = splitDeployments(deployments);
  const redeployment = useMutation({ mutationFn: () => redeploy(serviceId) });
  if (!service) {
    return null;
  }

  return (
    <div className="flex flex-col gap-6 p-6">
      <div className="flex flex-wrap items-center gap-3">
        {service.domains.map((domain) => (
          <a
            key={domain}
            href={domainUrl(domain)}
            target="_blank"
            rel="noreferrer"
            className="flex items-center gap-2 text-sm hover:underline"
          >
            <GlobeIcon className="size-4 text-muted-foreground" />
            {domain}
          </a>
        ))}
        <Button
          className="ms-auto"
          variant="outline"
          size="sm"
          disabled={!active}
          loading={redeployment.isPending}
          onClick={() => redeployment.mutate()}
        >
          <RotateCwIcon />
          Redéployer
        </Button>
      </div>
      {redeployment.error && (
        <p role="alert" className="text-destructive-foreground text-sm">
          {redeployment.error.message}
        </p>
      )}
      {active ? (
        <DeploymentCard service={service} deployment={active} />
      ) : (
        <p className="text-muted-foreground text-sm">Aucun déploiement actif.</p>
      )}
      {others.length > 0 && (
        <section className="flex flex-col gap-3">
          <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">
            Historique
          </h3>
          {others.map((deployment) => (
            <DeploymentCard key={deployment.id} service={service} deployment={deployment} />
          ))}
        </section>
      )}
    </div>
  );
}
