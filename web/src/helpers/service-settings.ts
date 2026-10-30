import type { DatabaseKind, ServiceSettings, ServiceSource } from "~/api/types";

import {
  DecodeError,
  decodeRecord,
  readBoolean,
  readNullable,
  readNumber,
  readOneOf,
  readString,
} from "./decode";

const TEMPLATES: readonly DatabaseKind[] = ["postgres", "mongodb", "redis"];

export function parseServiceSettings(raw: unknown): ServiceSettings {
  const record = decodeRecord(raw, "settings");
  return {
    source: parseSource(record.source),
    startCommand: readNullable(record, "startCommand", "settings", readString),
    port: readNullable(record, "port", "settings", readNumber),
    healthcheckPath: readNullable(record, "healthcheckPath", "settings", readString),
    cpuLimit: readNumber(record, "cpuLimit", "settings"),
    memoryLimitMb: readNumber(record, "memoryLimitMb", "settings"),
    egress: readBoolean(record, "egress", "settings"),
  };
}

function parseSource(raw: unknown): ServiceSource {
  const record = decodeRecord(raw, "source");
  switch (record.kind) {
    case "github":
      return {
        kind: "github",
        repository: readString(record, "repository", "source"),
        branch: readString(record, "branch", "source"),
        autoDeploy: readBoolean(record, "autoDeploy", "source"),
      };
    case "image":
      return { kind: "image", image: readString(record, "image", "source") };
    case "template":
      return {
        kind: "template",
        template: readOneOf(record, "template", TEMPLATES, "source"),
        version: readString(record, "version", "source"),
      };
    default:
      throw new DecodeError("source.kind: expected github, image or template");
  }
}
