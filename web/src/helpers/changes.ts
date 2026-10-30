import type {
  ChangeId,
  EnvironmentId,
  ServiceId,
  ServiceSettings,
  SettingField,
  SettingValue,
  StagedChange,
  Variable,
  VariableKind,
  VariableLevel,
} from "~/api/types";

import {
  DecodeError,
  decodeArray,
  decodeRecord,
  readNullable,
  readOneOf,
  readString,
} from "./decode";
import { parseImageReference } from "./new-project";
import { domainUrl, formatCpu, formatMemory } from "./services";
import type { ResolvedVariable } from "./variables";

const FIELDS: readonly SettingField[] = [
  "repository",
  "branch",
  "autoDeploy",
  "image",
  "startCommand",
  "port",
  "healthcheckPath",
  "cpuLimit",
  "memoryLimitMb",
  "egress",
  "domains",
];
const KINDS: readonly VariableKind[] = ["plain", "build", "secret"];

const FIELD_LABELS: Record<SettingField, string> = {
  repository: "Dépôt",
  branch: "Branche",
  autoDeploy: "Déploiement automatique",
  image: "Image",
  startCommand: "Commande de démarrage",
  port: "Port",
  healthcheckPath: "Healthcheck",
  cpuLimit: "Limite CPU",
  memoryLimitMb: "Limite mémoire",
  egress: "Accès Internet sortant",
  domains: "Domaines publics",
};

// The steps of the limit sliders.
export const CPU_LIMITS: readonly number[] = [0.25, 0.5, 1, 2, 4];
export const MEMORY_LIMITS_MB: readonly number[] = [256, 512, 1024, 2048, 4096, 8192];

// The index of the slider step for a limit: the closest step at or below it.
export function limitStep(steps: readonly number[], value: number): number {
  return Math.max(
    steps.findLastIndex((step) => step <= value),
    0,
  );
}

const MAX_COMMAND_LENGTH = 500;

export function parseChanges(raw: unknown): StagedChange[] {
  return decodeArray(raw, parseChange, "changes");
}

function parseChange(raw: unknown): StagedChange {
  const record = decodeRecord(raw, "change");
  const target = {
    id: readString(record, "id", "change") as ChangeId,
    environmentId: readString(record, "environmentId", "change") as EnvironmentId,
    serviceId: readString(record, "serviceId", "change") as ServiceId,
  };
  switch (record.type) {
    case "setting":
      return {
        ...target,
        type: "setting",
        field: readOneOf(record, "field", FIELDS, "change"),
        value: parseSettingValue(record.value),
      };
    case "variable":
      return {
        ...target,
        type: "variable",
        name: readString(record, "name", "change"),
        kind: readOneOf(record, "kind", KINDS, "change"),
        value: readNullable(record, "value", "change", readString),
      };
    case "variable-removal":
      return { ...target, type: "variable-removal", name: readString(record, "name", "change") };
    default:
      throw new DecodeError("change.type: expected setting, variable or variable-removal");
  }
}

function parseSettingValue(raw: unknown): SettingValue {
  if (typeof raw === "string" || typeof raw === "number" || typeof raw === "boolean") {
    return raw;
  }
  if (Array.isArray(raw) && raw.every((item) => typeof item === "string")) {
    return raw;
  }
  throw new DecodeError("change.value: expected a string, a number, a boolean or strings");
}

// The current value of a field, undefined when the field does not apply to this source.
export function settingValue(
  settings: ServiceSettings,
  domains: readonly string[],
  field: SettingField,
): SettingValue | undefined {
  const { source } = settings;
  switch (field) {
    case "repository":
      return source.kind === "github" ? source.repository : undefined;
    case "branch":
      return source.kind === "github" ? source.branch : undefined;
    case "autoDeploy":
      return source.kind === "github" ? source.autoDeploy : undefined;
    case "image":
      return source.kind === "image" ? source.image : undefined;
    case "domains":
      return domains;
    case "startCommand":
      return settings.startCommand ?? "";
    case "port":
      return settings.port ?? undefined;
    case "healthcheckPath":
      return settings.healthcheckPath ?? "";
    default:
      return settings[field];
  }
}

export function withStagedSettings(
  settings: ServiceSettings,
  domains: readonly string[],
  changes: readonly StagedChange[],
): { readonly settings: ServiceSettings; readonly domains: readonly string[] } {
  const staged = new Map<SettingField, SettingValue>();
  for (const change of changes) {
    if (change.type === "setting") {
      staged.set(change.field, change.value);
    }
  }
  const text = (field: SettingField, current: string) => {
    const value = staged.get(field);
    return typeof value === "string" ? value : current;
  };
  const optionalText = (field: SettingField, current: string | null) => {
    const value = staged.get(field);
    return typeof value === "string" ? value || null : current;
  };
  const number = (field: SettingField, current: number) => {
    const value = staged.get(field);
    return typeof value === "number" ? value : current;
  };
  const flag = (field: SettingField, current: boolean) => {
    const value = staged.get(field);
    return typeof value === "boolean" ? value : current;
  };
  const stagedDomains = staged.get("domains");
  const { source } = settings;

  return {
    settings: {
      ...settings,
      source:
        source.kind === "github"
          ? {
              ...source,
              repository: text("repository", source.repository),
              branch: text("branch", source.branch),
              autoDeploy: flag("autoDeploy", source.autoDeploy),
            }
          : source.kind === "image"
            ? { ...source, image: text("image", source.image) }
            : source,
      startCommand: optionalText("startCommand", settings.startCommand),
      port: settings.port === null ? null : number("port", settings.port),
      healthcheckPath: optionalText("healthcheckPath", settings.healthcheckPath),
      cpuLimit: number("cpuLimit", settings.cpuLimit),
      memoryLimitMb: number("memoryLimitMb", settings.memoryLimitMb),
      egress: flag("egress", settings.egress),
    },
    domains: Array.isArray(stagedDomains) ? stagedDomains : domains,
  };
}

export function stagedFields(changes: readonly StagedChange[]): ReadonlySet<SettingField> {
  return new Set(changes.flatMap((change) => (change.type === "setting" ? [change.field] : [])));
}

function sameValue(a: SettingValue | undefined, b: SettingValue | undefined): boolean {
  return Array.isArray(a) && Array.isArray(b) ? a.join("\n") === b.join("\n") : a === b;
}

// What editing a field means: stage the new value, drop a change that came back to the
// current value, or nothing at all.
export function settingChange(
  field: SettingField,
  value: SettingValue,
  current: SettingValue | undefined,
  changes: readonly StagedChange[],
):
  | { readonly action: "stage" }
  | { readonly action: "unstage"; readonly changeId: ChangeId }
  | { readonly action: "none" } {
  const existing = changes.find((change) => change.type === "setting" && change.field === field);
  if (sameValue(value, current)) {
    return existing ? { action: "unstage", changeId: existing.id } : { action: "none" };
  }
  if (existing?.type === "setting" && sameValue(existing.value, value)) {
    return { action: "none" };
  }
  return { action: "stage" };
}

type TextField = "image" | "startCommand" | "port" | "healthcheckPath";

export function validateSetting(
  field: TextField,
  raw: string,
): { readonly value: SettingValue } | { readonly error: string } {
  const text = raw.trim();
  switch (field) {
    case "port": {
      const port = Number(text);
      return /^\d+$/.test(text) && port >= 1 && port <= 65_535
        ? { value: port }
        : { error: "Un port entre 1 et 65535." };
    }
    case "healthcheckPath":
      return text === "" || /^\/\S{0,200}$/.test(text)
        ? { value: text }
        : { error: "Un chemin qui commence par /, sans espace." };
    case "image": {
      const image = parseImageReference(text);
      return image ? { value: image } : { error: "Référence d'image invalide." };
    }
    case "startCommand":
      return text.length <= MAX_COMMAND_LENGTH
        ? { value: text }
        : { error: `${MAX_COMMAND_LENGTH} caractères au plus.` };
  }
}

export function parseDomain(raw: string): string | undefined {
  const domain = raw.trim().toLowerCase();
  return domainUrl(domain) ? domain : undefined;
}

export function changeCountLabel(count: number): string {
  return `Appliquer ${count} ${count > 1 ? "changements" : "changement"}`;
}

function formatSetting(field: SettingField, value: SettingValue): string {
  if (typeof value === "boolean") {
    return value ? "activé" : "désactivé";
  }
  if (Array.isArray(value)) {
    return value.length === 0 ? "aucun" : value.join(", ");
  }
  if (field === "cpuLimit" && typeof value === "number") {
    return formatCpu(value);
  }
  if (field === "memoryLimitMb" && typeof value === "number") {
    return formatMemory(value);
  }
  return String(value) || "par défaut";
}

export function describeChange(change: StagedChange): string {
  switch (change.type) {
    case "setting":
      return `${FIELD_LABELS[change.field]} → ${formatSetting(change.field, change.value)}`;
    case "variable":
      return `${change.name} = ${change.value ?? "••••••••"}`;
    case "variable-removal":
      return `${change.name} supprimée`;
  }
}

export interface VariableDraftRow {
  readonly name: string;
  readonly kind: VariableKind;
  // Null for a secret.
  readonly value: string | null;
  readonly origin: VariableLevel;
  // The applied variable, absent for one the draft adds.
  readonly variable: Variable | undefined;
  readonly draft: "added" | "modified" | "removed" | undefined;
  readonly changeId: ChangeId | undefined;
}

// The variables of a service as they will be after the deploy, each marked by its draft.
export function stagedVariableRows(
  resolved: readonly ResolvedVariable[],
  changes: readonly StagedChange[],
): VariableDraftRow[] {
  const byName = new Map<string, StagedChange>();
  for (const change of changes) {
    if (change.type === "variable" || change.type === "variable-removal") {
      byName.set(change.name, change);
    }
  }

  const rows: VariableDraftRow[] = resolved.map(({ variable, origin }) => {
    const change = byName.get(variable.name);
    byName.delete(variable.name);
    if (change?.type === "variable") {
      return {
        name: variable.name,
        kind: change.kind,
        value: change.value,
        origin: "service",
        variable,
        draft: "modified",
        changeId: change.id,
      };
    }
    return {
      name: variable.name,
      kind: variable.kind,
      value: variable.value,
      origin,
      variable,
      draft: change ? "removed" : undefined,
      changeId: change?.id,
    };
  });

  for (const change of byName.values()) {
    if (change.type === "variable") {
      rows.push({
        name: change.name,
        kind: change.kind,
        value: change.value,
        origin: "service",
        variable: undefined,
        draft: "added",
        changeId: change.id,
      });
    }
  }
  return rows.toSorted((a, b) => a.name.localeCompare(b.name));
}

export function changesByService<S extends { readonly id: string }>(
  changes: readonly StagedChange[],
  services: readonly S[],
): { readonly service: S; readonly changes: StagedChange[] }[] {
  return services
    .map((service) => ({
      service,
      changes: changes.filter((change) => change.serviceId === service.id),
    }))
    .filter((group) => group.changes.length > 0);
}
