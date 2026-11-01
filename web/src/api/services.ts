import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { queryOptions } from "@tanstack/react-query";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseServiceSettings } from "~/helpers/service-settings";
import { isInFlight } from "~/helpers/deployments";
import { parseService, parseServices } from "~/helpers/services";

import { apiDelete, apiGet, apiPost } from "./client";
import { queryClient } from "./query-client";
import type { EnvironmentId, Service, ServiceId } from "./types";

// Nothing pushes a status change to the panel: poll while a service is deploying.
export const IN_FLIGHT_POLL_MS = 2_000;

export const servicesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["services"],
    queryFn: () => apiGet("/api/services", parseServices),
    queryClient,
    getKey: (service) => service.id,
    refetchInterval: (query) =>
      query.state.data?.some((service) => isInFlight(service.status)) ? IN_FLIGHT_POLL_MS : false,
  }),
);

export const allServices = (q: InitialQueryBuilder) => q.from({ service: servicesCollection });

export const environmentServices = (environmentId: EnvironmentId) => (q: InitialQueryBuilder) =>
  q
    .from({ service: servicesCollection })
    .where(({ service }) => eq(service.environmentId, environmentId));

export const serviceById = (serviceId: string) => (q: InitialQueryBuilder) =>
  q
    .from({ service: servicesCollection })
    .where(({ service }) => eq(service.id, serviceId))
    .findOne();

export const serviceSettingsQuery = (serviceId: ServiceId) =>
  queryOptions({
    queryKey: ["services", serviceId, "settings"],
    queryFn: () => apiGet(`/api/services/${serviceId}/settings`, parseServiceSettings),
  });

export async function deleteService(serviceId: ServiceId): Promise<void> {
  await apiDelete(`/api/services/${serviceId}`);
  await servicesCollection.utils.refetch();
}

// The server resolves the image, then deploys it: the service comes back `deploying`.
export async function createService(
  environmentId: EnvironmentId,
  input: { readonly image: string; readonly name?: string },
): Promise<Service> {
  const service = await apiPost(`/api/environments/${environmentId}/services`, input, parseService);
  await Promise.all([
    servicesCollection.utils.refetch(),
    queryClient.invalidateQueries({ queryKey: ["deployments"] }),
  ]);
  return service;
}
