import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection } from "@tanstack/react-db";

import { decodeRecord, readString } from "~/helpers/decode";
import { parseVariables } from "~/helpers/variables";

import { apiDelete, apiGet, apiPost } from "./client";
import { queryClient } from "./query-client";
import type { NewVariable, VariableId } from "./types";

// Secret values never enter this collection: the API sends them as null.
export const variablesCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["variables"],
    queryFn: () => apiGet("/api/variables", parseVariables),
    queryClient,
    getKey: (variable) => variable.id,
    onDelete: async ({ transaction }) => {
      await Promise.all(
        transaction.mutations.map((mutation) => apiDelete(`/api/variables/${mutation.key}`)),
      );
    },
  }),
);

export const allVariables = (q: InitialQueryBuilder) => q.from({ variable: variablesCollection });

export async function createVariable(variable: NewVariable): Promise<void> {
  await apiPost("/api/variables", variable, () => undefined);
  await variablesCollection.utils.refetch();
}

// Optimistic: the row disappears at once and comes back if the server refuses.
export function deleteVariable(variableId: VariableId): void {
  variablesCollection.delete(variableId);
}

// A reveal is a POST the server audits. Its result lives in the caller's state only,
// never in a cache.
export function revealSecret(variableId: VariableId): Promise<string> {
  return apiPost(`/api/variables/${variableId}/reveal`, {}, (raw) =>
    readString(decodeRecord(raw, "secret"), "value", "secret"),
  );
}
