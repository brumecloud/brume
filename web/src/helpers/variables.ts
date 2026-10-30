import type {
  Environment,
  Service,
  Variable,
  VariableId,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import { decodeArray, decodeRecord, readNullable, readOneOf, readString } from "./decode";
import { isDatabase } from "./services";

const LEVELS: readonly VariableLevel[] = ["project", "environment", "service"];
const KINDS: readonly VariableKind[] = ["plain", "build", "secret"];
const NAME = /^[A-Za-z_][A-Za-z0-9_]{0,127}$/;
const SECRET_NAME = /(?:_KEY|_TOKEN|_SECRET)$|PASSWORD/i;
const CREDENTIALS_IN_URL = /^[a-z][a-z0-9+.-]*:\/\/[^/\s:@]+:[^/\s@]+@/i;

export interface ResolvedVariable {
  readonly variable: Variable;
  readonly origin: VariableLevel;
}

export function parseVariables(raw: unknown): Variable[] {
  return decodeArray(raw, parseVariable, "variables");
}

function parseVariable(raw: unknown): Variable {
  const record = decodeRecord(raw, "variable");
  return {
    id: readString(record, "id", "variable") as VariableId,
    level: readOneOf(record, "level", LEVELS, "variable"),
    ownerId: readString(record, "ownerId", "variable"),
    name: readString(record, "name", "variable"),
    kind: readOneOf(record, "kind", KINDS, "variable"),
    value: readNullable(record, "value", "variable", readString),
  };
}

// archi/base/13-modele-de-donnees.md: the most precise level wins (service, then environment,
// then project). Shared secrets only reach the services that reference them, a template
// database only takes its own settings, a static site only the shared build variables.
export function resolveVariables(
  variables: readonly Variable[],
  service: Service,
  environment: Environment,
): ResolvedVariable[] {
  const sharedKinds: readonly VariableKind[] = isDatabase(service.kind)
    ? []
    : service.kind === "static"
      ? ["build"]
      : ["plain", "build"];
  const owners: Record<VariableLevel, string> = {
    project: environment.projectId,
    environment: environment.id,
    service: service.id,
  };

  const byName = new Map<string, ResolvedVariable>();
  for (const level of LEVELS) {
    for (const variable of variables) {
      const reaches =
        variable.ownerId === owners[level] &&
        variable.level === level &&
        (level === "service" || sharedKinds.includes(variable.kind));
      if (reaches) {
        byName.set(variable.name, { variable, origin: level });
      }
    }
  }
  return [...byName.values()].toSorted((a, b) => a.variable.name.localeCompare(b.variable.name));
}

export function parseVariableName(raw: string): string | undefined {
  return NAME.test(raw) ? raw : undefined;
}

// A plain or build value is kept in the revision history: warn before it holds a secret.
export function variableWarnings(name: string, value: string, kind: VariableKind): string[] {
  if (kind === "secret") {
    return [];
  }
  return [
    ...(SECRET_NAME.test(name)
      ? [
          "Ce nom ressemble à celui d'un secret : sa valeur resterait dans l'historique des révisions.",
        ]
      : []),
    ...(CREDENTIALS_IN_URL.test(value)
      ? ["Cette valeur contient des identifiants : faites-en plutôt un secret."]
      : []),
  ];
}

export function variableCountLabel(count: number): string {
  if (count === 0) {
    return "Aucune variable";
  }
  return `${count} ${count > 1 ? "variables" : "variable"}`;
}

export function parseVariableKind(raw: unknown): VariableKind | undefined {
  return KINDS.find((kind) => kind === raw);
}
