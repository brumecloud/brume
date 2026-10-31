import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection, eq } from "@tanstack/react-db";

import { parseChanges } from "~/helpers/changes";

import { apiDelete, apiGet, apiPost } from "./client";
import { deploymentsCollection } from "./deployments";
import { queryClient } from "./query-client";
import { servicesCollection } from "./services";
import type { ChangeId, EnvironmentId, NewChange } from "./types";
import { variablesCollection } from "./variables";

// Staged secret values never enter this collection: the API sends them as null.
export const changesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["changes"],
    queryFn: () => apiGet("/api/changes", parseChanges),
    queryClient,
    getKey: (change) => change.id,
    onDelete: async ({ transaction }) => {
      await Promise.all(
        transaction.mutations.map((mutation) => apiDelete(`/api/changes/${mutation.key}`)),
      );
    },
  }),
);

export const environmentChanges = (environmentId: EnvironmentId) => (q: InitialQueryBuilder) =>
  q
    .from({ change: changesCollection })
    .where(({ change }) => eq(change.environmentId, environmentId));

export const serviceChanges = (serviceId: string) => (q: InitialQueryBuilder) =>
  q.from({ change: changesCollection }).where(({ change }) => eq(change.serviceId, serviceId));

// Not optimistic: the value of a staged secret must not pass through the collection.
export async function stageChange(change: NewChange): Promise<void> {
  await apiPost("/api/changes", change, () => undefined);
  await changesCollection.utils.refetch();
}

// Optimistic: the draft disappears at once and comes back if the server refuses.
export function unstageChange(changeId: ChangeId): void {
  changesCollection.delete(changeId);
}

// One deploy for every staged change of the environment: new revisions, then deployments.
export async function deployChanges(environmentId: EnvironmentId): Promise<void> {
  await apiPost(`/api/environments/${environmentId}/changes/deploy`, {}, () => undefined);
  await Promise.all([
    changesCollection.utils.refetch(),
    servicesCollection.utils.refetch(),
    deploymentsCollection.utils.refetch(),
    variablesCollection.utils.refetch(),
    queryClient.invalidateQueries({ queryKey: ["services"] }),
  ]);
}

export async function discardChanges(environmentId: EnvironmentId): Promise<void> {
  await apiDelete(`/api/environments/${environmentId}/changes`);
  await changesCollection.utils.refetch();
}
