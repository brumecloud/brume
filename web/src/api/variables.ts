import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { queryOptions } from "@tanstack/react-query";
import { type InitialQueryBuilder, createCollection } from "@tanstack/react-db";

import { decodeRecord, readString } from "~/helpers/decode";
import { parseImpact, parseRemoval, parseSharedVariableResult } from "~/helpers/shared-variables";
import { parseVariables } from "~/helpers/variables";

import { apiDelete, apiGet, apiPost } from "./client";
import { deploymentsCollection } from "./deployments";
import { queryClient } from "./query-client";
import { servicesCollection } from "./services";
import type {
  Deployment,
  SharedVariableLevel,
  SharedVariableResult,
  VariableId,
  VariableImpact,
  VariableKind,
} from "./types";

// The applied variables. Edits go through staged changes (api/changes.ts). Secret values never
// enter this collection: the API sends them as null.
export const variablesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["variables"],
    queryFn: () => apiGet("/api/variables", parseVariables),
    queryClient,
    getKey: (variable) => variable.id,
  }),
);

export const allVariables = (q: InitialQueryBuilder) => q.from({ variable: variablesCollection });

// A reveal is a POST the server audits. Its result lives in the caller's state only,
// never in a cache.
export function revealSecret(variableId: VariableId): Promise<string> {
  return apiPost(`/api/variables/${variableId}/reveal`, {}, (raw) =>
    readString(decodeRecord(raw, "secret"), "value", "secret"),
  );
}

// The services a shared variable reaches, before the admin confirms a change.
export const variableImpactQuery = (level: SharedVariableLevel, ownerId: string, name: string) =>
  queryOptions({
    queryKey: ["variables", "impact", level, ownerId, name],
    queryFn: () => variableImpact(level, ownerId, name),
    staleTime: 0,
  });

function variableImpact(
  level: SharedVariableLevel,
  ownerId: string,
  name: string,
): Promise<VariableImpact> {
  const query = new URLSearchParams({ level, ownerId, name });
  return apiGet(`/api/variables/impact?${query}`, parseImpact);
}

// Applied at once, not staged: a plain or build value redeploys the services it reaches, a secret
// waits for their next deployment. Not optimistic, for the value of a secret.
export async function setSharedVariable(input: {
  readonly level: SharedVariableLevel;
  readonly ownerId: string;
  readonly name: string;
  readonly kind: VariableKind;
  readonly value: string;
}): Promise<SharedVariableResult> {
  const result = await apiPost("/api/variables", input, parseSharedVariableResult);
  await refreshAfterDeploy();
  return result;
}

// Refused (409) while an active deployment references the secret.
export async function removeSharedVariable(
  variableId: VariableId,
): Promise<{ readonly deployments: readonly Deployment[] }> {
  const result = await apiDelete(`/api/variables/${variableId}`, parseRemoval);
  await refreshAfterDeploy();
  return result;
}

async function refreshAfterDeploy(): Promise<void> {
  await Promise.all([
    variablesCollection.utils.refetch(),
    deploymentsCollection.utils.refetch(),
    servicesCollection.utils.refetch(),
  ]);
}
