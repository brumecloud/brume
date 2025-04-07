import { z } from "zod";

/**
 * @deprecated use direct validation from graphql
 */
export const DockerRunnerSchema = z.object({
  type: z.literal("generic-docker"),
  data: z.object({
    command: z.string().nullable(),
    healthCheckURL: z.string().url({
      message: "Invalid URL",
    }),
    memory: z.object({
      limit: z
        .number()
        .min(1, {
          message: "Memory limit must be greater than 1Mb",
        })
        .max(1000, {
          message: "Memory limit must be less than 1000Mb",
        })
        .default(100),
      request: z
        .number()
        .min(1, {
          message: "Memory request must be greater than 1Mb",
        })
        .max(1000, {
          message: "Memory request must be less than 1000Mb",
        })
        .default(100),
    }),
    cpu: z.object({
      limit: z
        .number()
        .min(0.05, {
          message: "CPU limit must be greater than 0.05",
        })
        .max(2, {
          message: "CPU limit must be less than 2",
        })
        .default(0.2),
      request: z
        .number()
        .min(0.05, {
          message: "CPU request must be greater than 0.05",
        })
        .max(2, {
          message: "CPU request must be less than 2",
        })
        .default(0.2),
    }),
    port: z.number().min(1).max(65535).default(80),
    publicDomain: z
      .string()
      .regex(/^[a-z0-9-]+$/, {
        message: "Invalid public domain",
      })
      .or(z.literal("")),
    privateDomain: z
      .string()
      .regex(/^[a-z0-9-]+$/, {
        message: "Invalid private domain",
      })
      .or(z.literal("")),
  }),
});

/**
 * @deprecated use validation from graphql
 */
export const RunnerSchema = z.discriminatedUnion("type", [
  DockerRunnerSchema,
]);

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Runner = z.infer<typeof RunnerSchema>;

/**
 * @deprecated use direct validation from graphql
 */
export const GenericDockerImageBuilderSchema = z.object({
  type: z.literal("generic-docker"),
  data: z.object({
    image: z.string(),
    registry: z.string(),
    tag: z.string(),
  }),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export const BuilderSchema = z.discriminatedUnion("type", [
  GenericDockerImageBuilderSchema,
]);

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Builder = z.infer<typeof BuilderSchema>;

/**
 * @deprecated use direct validation from graphql
 */
export const DeploymentSchema = z.object({
  __typename: z.literal("Deployment"),
  id: z.string(),
  createdAt: z.string(),
  env: z.string(),
  source: z.discriminatedUnion("type", [
    z.object({
      type: z.literal("git"),
      branch: z.string(),
      commit: z.string(),
      message: z.string(),
    }),
    z.object({
      type: z.literal("console"),
    }),
  ]),
  logs: z.object({
    status: z.enum(["success", "pending", "failed"]),
    date: z.string(),
    duration: z.coerce.number(),
  }),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Deployment = z.infer<typeof DeploymentSchema>;

/**
 * @deprecated use direct validation from graphql
 */
export const ServiceSchema = z.object({
  __typename: z.literal("Service"),
  name: z.string(),
  id: z.string(),
  liveBuilder: BuilderSchema.nullable(),
  liveRunner: RunnerSchema.nullable(),
  draftRunner: RunnerSchema.nullable(),
  draftBuilder: BuilderSchema.nullable(),
  deployments: z.array(DeploymentSchema),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Service = z.infer<typeof ServiceSchema>;
