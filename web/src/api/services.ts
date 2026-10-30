import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseServices } from "~/helpers/services";

import { apiGet } from "./client";
import { queryClient } from "./query-client";
import type { EnvironmentId } from "./types";

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
