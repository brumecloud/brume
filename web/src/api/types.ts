// Mirrors archi/base/13-modele-de-donnees.md.

declare const brand: unique symbol;
type Id<Name extends string> = string & { readonly [brand]: Name };

export type ProjectId = Id<"ProjectId">;
export type EnvironmentId = Id<"EnvironmentId">;

export interface Project {
  readonly id: ProjectId;
  readonly name: string;
  readonly description: string;
  readonly updatedAt: string;
}

export interface Environment {
  readonly id: EnvironmentId;
  readonly projectId: ProjectId;
  // A DNS label, also used as the URL segment (`/projects/…/production`).
  readonly name: string;
}

export type ServiceId = Id<"ServiceId">;

// Fixed at creation: it decides what the service can do.
export type ServiceKind = "github" | "image" | "static" | "postgres" | "mongodb" | "redis";

export type DatabaseKind = Extract<ServiceKind, "postgres" | "mongodb" | "redis">;

// Computed by the server from the latest deployment.
export type ServiceStatus = "online" | "deploying" | "crashed" | "stopped";

export interface Volume {
  readonly name: string;
  readonly mountPath: string;
}

export interface Service {
  readonly id: ServiceId;
  readonly environmentId: EnvironmentId;
  readonly name: string;
  // Immutable DNS name on the environment network.
  readonly slug: string;
  readonly kind: ServiceKind;
  readonly status: ServiceStatus;
  readonly domains: readonly string[];
  readonly volume: Volume | null;
  // Services this one connects to, derived by the server from the secrets it references.
  readonly dependencies: readonly ServiceId[];
}

export interface Repository {
  // `owner/name`, as GitHub spells it.
  readonly fullName: string;
}

export type NewProjectSource =
  | { readonly kind: "empty" }
  | { readonly kind: "github" | "static"; readonly repository: string }
  | { readonly kind: DatabaseKind }
  | { readonly kind: "image"; readonly image: string };

export interface CreatedProject {
  readonly projectId: ProjectId;
  readonly environmentName: string;
}

export type DeploymentId = Id<"DeploymentId">;

// `superseded`: replaced by a later deployment. History only grows, a rollback is a new deployment.
export type DeploymentStatus = "building" | "deploying" | "active" | "failed" | "superseded";

export interface Commit {
  readonly sha: string;
  readonly message: string;
  readonly branch: string;
}

export interface Deployment {
  readonly id: DeploymentId;
  readonly serviceId: ServiceId;
  readonly status: DeploymentStatus;
  readonly createdAt: string;
  // `GitHub` for a push, otherwise the Tailscale login of the admin.
  readonly author: string;
  readonly commit: Commit | null;
  readonly image: string | null;
  // The engine keeps the artifacts of the last 10 revisions only.
  readonly rollbackable: boolean;
}

export type VariableId = Id<"VariableId">;
export type VariableKind = "plain" | "build" | "secret";
export type VariableLevel = "project" | "environment" | "service";

export interface Variable {
  readonly id: VariableId;
  readonly level: VariableLevel;
  // Id of the project, environment or service that holds the variable.
  readonly ownerId: string;
  readonly name: string;
  readonly kind: VariableKind;
  // Always null for a secret: the API only sends it through an explicit, audited reveal.
  readonly value: string | null;
}

export type ServiceSource =
  | {
      readonly kind: "github";
      readonly repository: string;
      readonly branch: string;
      readonly autoDeploy: boolean;
    }
  | { readonly kind: "image"; readonly image: string }
  | { readonly kind: "template"; readonly template: DatabaseKind; readonly version: string };

export interface ServiceSettings {
  readonly source: ServiceSource;
  readonly startCommand: string | null;
  readonly port: number | null;
  readonly healthcheckPath: string | null;
  readonly cpuLimit: number;
  readonly memoryLimitMb: number;
  // Outbound internet access, off by default for databases.
  readonly egress: boolean;
}

export type LogLevel = "info" | "warn" | "error";

// The message is hostile: apps copy requests from the outside into their logs.
export interface LogLine {
  readonly id: string;
  readonly timestamp: string;
  readonly serviceId: ServiceId;
  readonly level: LogLevel;
  readonly message: string;
}

export type DeploymentLogStream = "build" | "deploy";

export interface RevisionVariable {
  readonly name: string;
  readonly kind: VariableKind;
  // Plain and build values are part of the revision; a secret only by its name.
  readonly value: string | null;
  readonly origin: VariableLevel;
}

// The immutable spec a deployment put in service. See archi/base/13-modele-de-donnees.md.
export interface Revision {
  readonly digest: string;
  readonly builder: string | null;
  readonly startCommand: string | null;
  readonly variables: readonly RevisionVariable[];
}

// Staged changes: edited in the panel, applied together by one deploy, like Railway.
// They live on the server, so a staged secret never sits in the browser.
export type SettingField =
  | "branch"
  | "autoDeploy"
  | "image"
  | "startCommand"
  | "port"
  | "healthcheckPath"
  | "cpuLimit"
  | "memoryLimitMb"
  | "egress"
  | "domains";

export type SettingValue = string | number | boolean | readonly string[];

export type ChangeId = Id<"ChangeId">;

interface ChangeTarget {
  readonly id: ChangeId;
  readonly environmentId: EnvironmentId;
  readonly serviceId: ServiceId;
}

export type StagedChange =
  | (ChangeTarget & {
      readonly type: "setting";
      readonly field: SettingField;
      readonly value: SettingValue;
    })
  | (ChangeTarget & {
      readonly type: "variable";
      readonly name: string;
      readonly kind: VariableKind;
      // Null for a secret: the server keeps the value until the deploy.
      readonly value: string | null;
    })
  | (ChangeTarget & { readonly type: "variable-removal"; readonly name: string });

// What the panel sends: the server finds the environment and assigns the id.
export type NewChange =
  | {
      readonly serviceId: ServiceId;
      readonly type: "setting";
      readonly field: SettingField;
      readonly value: SettingValue;
    }
  | {
      readonly serviceId: ServiceId;
      readonly type: "variable";
      readonly name: string;
      readonly kind: VariableKind;
      readonly value: string;
    }
  | { readonly serviceId: ServiceId; readonly type: "variable-removal"; readonly name: string };
