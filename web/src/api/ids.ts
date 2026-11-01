// Branded identifiers: a ProjectId cannot stand where a ServiceId is expected. The generated
// declarations import them from here.

declare const brand: unique symbol;
type Id<Name extends string> = string & { readonly [brand]: Name };

export type ProjectId = Id<"ProjectId">;
export type EnvironmentId = Id<"EnvironmentId">;
export type ServiceId = Id<"ServiceId">;
export type DeploymentId = Id<"DeploymentId">;
export type VariableId = Id<"VariableId">;
export type ChangeId = Id<"ChangeId">;
