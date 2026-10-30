import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseDeployments } from "~/helpers/deployments";

import { apiGet, apiPost } from "./client";
import { queryClient } from "./query-client";
import { servicesCollection } from "./services";
import type { DeploymentId, ServiceId } from "./types";

export const deploymentsCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["deployments"],
    queryFn: () => apiGet("/api/deployments", parseDeployments),
    queryClient,
    getKey: (deployment) => deployment.id,
  }),
);

export const serviceDeployments = (serviceId: ServiceId) => (q: InitialQueryBuilder) =>
  q
    .from({ deployment: deploymentsCollection })
    .where(({ deployment }) => eq(deployment.serviceId, serviceId));

// A deployment changes the status of its service: refetch both.
async function refresh(): Promise<void> {
  await Promise.all([deploymentsCollection.utils.refetch(), servicesCollection.utils.refetch()]);
}

export async function redeploy(serviceId: ServiceId): Promise<void> {
  await apiPost(`/api/services/${serviceId}/redeploy`, {}, () => undefined);
  await refresh();
}

// Puts an old revision back in service, by its digest, without a rebuild.
export async function rollback(deploymentId: DeploymentId): Promise<void> {
  await apiPost(`/api/deployments/${deploymentId}/rollback`, {}, () => undefined);
  await refresh();
}
