import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { type InitialQueryBuilder, createCollection } from "@tanstack/react-db";

import { decodeRecord, readString } from "~/helpers/decode";
import { parseVariables } from "~/helpers/variables";

import { apiGet, apiPost } from "./client";
import { queryClient } from "./query-client";
import type { VariableId } from "./types";

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
