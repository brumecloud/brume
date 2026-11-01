// The API's shapes. Most are generated from the Rust DTOs (`generated.ts`, see
// crates/brume-api/src/dto_ts.rs); this file keeps by hand only those without a Rust equivalent.
// Mirrors archi/base/13-modele-de-donnees.md.

import type { VariableLevel } from "./generated";

export type {
  ChangeId,
  DeploymentId,
  EnvironmentId,
  ProjectId,
  ServiceId,
  VariableId,
} from "./ids";

export type {
  ActivityEvent,
  ActivityPage,
  Commit,
  CreatedProject,
  DatabaseKind,
  Deployment,
  DeploymentLogStream,
  DeploymentStatus,
  Environment,
  LogLevel,
  LogLine,
  NewChange,
  NewProjectSource,
  Project,
  RevisionVariable,
  Revision,
  Service,
  ServiceKind,
  ServiceSettings,
  ServiceSource,
  ServiceStatus,
  SettingField,
  SettingValue,
  SharedVariableResult,
  StagedChange,
  Variable,
  VariableImpact,
  VariableKind,
  VariableLevel,
  Volume,
} from "./generated";

// A GitHub repository, for the sources that are not available yet.
export interface Repository {
  // `owner/name`, as GitHub spells it.
  readonly fullName: string;
}

export type SharedVariableLevel = Extract<VariableLevel, "project" | "environment">;
