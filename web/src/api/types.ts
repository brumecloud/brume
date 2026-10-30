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
