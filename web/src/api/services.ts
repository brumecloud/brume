import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection } from "@tanstack/react-db";

import { parseServices } from "~/helpers/services";

import { apiGet } from "./client";
import { queryClient } from "./query-client";

export const servicesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["services"],
    queryFn: () => apiGet("/api/services", parseServices),
    queryClient,
    getKey: (service) => service.id,
  }),
);

export const allServices = (q: InitialQueryBuilder) => q.from({ service: servicesCollection });
