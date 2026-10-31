import type {
  Environment,
  EnvironmentId,
  Service,
  ServiceId,
  SharedVariableResult,
  Variable,
  SharedVariableLevel,
  VariableImpact,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import { decodeArray, decodeRecord, readString, readStringArray } from "./decode";
import { parseDeployments } from "./deployments";
import { parseVariable, variableWarnings } from "./variables";

const PRODUCTION = "production";
const PROJECT_SECRET_REMINDER =
  "Project secrets also reach staging. Consider defining it per environment.";

export interface ImpactGroup {
  readonly environmentName: string;
  readonly services: string[];
}

// archi/base/13-modele-de-donnees.md#modifier-une-valeur-partagée: the services a change touches,
// environment by environment, production first.
export function groupImpact(
  services: VariableImpact["services"],
  environments: readonly Environment[],
  serviceRows: readonly Service[],
): ImpactGroup[] {
  const groups = new Map<string, string[]>();
  for (const { serviceId, environmentId } of services) {
    const environment = environments.find(({ id }) => id === environmentId);
    const service = serviceRows.find(({ id }) => id === serviceId);
    if (environment && service) {
      groups.set(environment.name, [...(groups.get(environment.name) ?? []), service.name]);
    }
  }
  return [...groups]
    .map(([environmentName, names]) => ({
      environmentName,
      services: names.toSorted((a, b) => a.localeCompare(b)),
    }))
    .toSorted(
      (a, b) =>
        Number(b.environmentName === PRODUCTION) - Number(a.environmentName === PRODUCTION) ||
        a.environmentName.localeCompare(b.environmentName),
    );
}

export function impactSummary(count: number): string {
  if (count === 0) {
    return "This change redeploys no services.";
  }
  return `This change redeploys ${count} ${count === 1 ? "service" : "services"}.`;
}

export function impactConfirmLabel(count: number): string {
  return count === 0 ? "Save" : `Redeploy ${count} ${count === 1 ? "service" : "services"}`;
}

// The services that reference a secret, located for `groupImpact`.
export function referencesImpact(
  serviceIds: readonly ServiceId[],
  serviceRows: readonly Service[],
): VariableImpact["services"] {
  return serviceRows
    .filter((service) => serviceIds.includes(service.id))
    .map((service) => ({ serviceId: service.id, environmentId: service.environmentId }));
}

// archi/base/13-modele-de-donnees.md: a project secret also reaches staging, built from branches
// reviewed less carefully.
export function sharedVariableWarnings(
  level: SharedVariableLevel,
  name: string,
  value: string,
  kind: VariableKind,
): string[] {
  return [
    ...variableWarnings(name, value, kind),
    ...(level === "project" && kind === "secret" ? [PROJECT_SECRET_REMINDER] : []),
  ];
}

// The variables one project or environment holds.
export function sharedVariables(
  variables: readonly Variable[],
  level: VariableLevel,
  ownerId: string,
): Variable[] {
  return variables
    .filter((variable) => variable.level === level && variable.ownerId === ownerId)
    .toSorted((a, b) => a.name.localeCompare(b.name));
}

// The shared secrets a service of the environment can reference, in resolution order: the
// environment's, then the project's.
export function referenceableSecrets(
  variables: readonly Variable[],
  environment: Environment,
  taken: readonly string[],
): string[] {
  const secrets = (level: VariableLevel, ownerId: string) =>
    sharedVariables(variables, level, ownerId)
      .filter((variable) => variable.kind === "secret")
      .map((variable) => variable.name);
  const names = [
    ...secrets("environment", environment.id),
    ...secrets("project", environment.projectId),
  ];
  return [...new Set(names)].filter((name) => !taken.includes(name));
}

export function parseSharedVariableResult(raw: unknown): SharedVariableResult {
  const record = decodeRecord(raw, "shared variable");
  return {
    variable: parseVariable(record.variable),
    deployments: parseDeployments(record.deployments),
    referencedBy: readStringArray(record, "referencedBy", "shared variable") as ServiceId[],
  };
}

export function parseRemoval(raw: unknown): Pick<SharedVariableResult, "deployments"> {
  return { deployments: parseDeployments(decodeRecord(raw, "removal").deployments) };
}

export function parseImpact(raw: unknown): VariableImpact {
  const record = decodeRecord(raw, "impact");
  return {
    services: decodeArray(
      record.services,
      (item) => {
        const service = decodeRecord(item, "impact service");
        return {
          serviceId: readString(service, "serviceId", "impact service") as ServiceId,
          environmentId: readString(service, "environmentId", "impact service") as EnvironmentId,
        };
      },
      "impact services",
    ),
  };
}
