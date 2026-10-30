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

export interface Service {
  readonly id: ServiceId;
  readonly environmentId: EnvironmentId;
  readonly name: string;
  // Immutable DNS name on the environment network.
  readonly slug: string;
  readonly kind: ServiceKind;
  readonly status: ServiceStatus;
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
