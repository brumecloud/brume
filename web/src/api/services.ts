import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { queryOptions } from "@tanstack/react-query";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseServiceSettings } from "~/helpers/service-settings";
import { parseServices } from "~/helpers/services";

import { apiDelete, apiGet } from "./client";
import { queryClient } from "./query-client";
import type { EnvironmentId, ServiceId } from "./types";

export const servicesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["services"],
    queryFn: () => apiGet("/api/services", parseServices),
    queryClient,
    getKey: (service) => service.id,
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
