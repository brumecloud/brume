import { z } from "zod";

export const DockerRunnerSchema = z.object({
  type: z.literal("generic-docker"),
  data: z.object({
    command: z.string(),
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
        }),
      request: z
        .number()
        .min(1, {
          message: "Memory request must be greater than 1Mb",
        })
        .max(1000, {
          message: "Memory request must be less than 1000Mb",
        }),
    }),
    cpu: z.object({
      limit: z
        .number()
        .min(0.05, {
          message: "CPU limit must be greater than 0.05",
        })
        .max(2, {
          message: "CPU limit must be less than 2",
        }),
      request: z
        .number()
        .min(0.05, {
          message: "CPU request must be greater than 0.05",
        })
        .max(2, {
          message: "CPU request must be less than 2",
        }),
    }),
    port: z.number().min(1).max(65535),
    publicDomain: z.string().regex(/^[a-z0-9-]+$/, {
      message: "Invalid public domain",
    }),
    privateDomain: z.string().regex(/^[a-z0-9-]+$/, {
      message: "Invalid private domain",
    }),
  }),
});

export const RunnerSchema = z.discriminatedUnion("type", [
  DockerRunnerSchema,
]);

export type Runner = z.infer<typeof RunnerSchema>;

export const GenericDockerImageBuilderSchema = z.object({
  type: z.literal("generic-docker"),
  data: z.object({
    image: z.string(),
    registry: z.string(),
    tag: z.string(),
  }),
});

export const BuilderSchema = z.discriminatedUnion("type", [
  GenericDockerImageBuilderSchema,
]);

export type Builder = z.infer<typeof BuilderSchema>;

export const ServiceSchema = z.object({
  __typename: z.literal("Service"),
  name: z.string(),
  id: z.string(),
  liveBuilder: BuilderSchema.nullable(),
  liveRunner: RunnerSchema.nullable(),
  draftRunner: RunnerSchema.nullable(),
  draftBuilder: BuilderSchema.nullable(),
});

export type Service = z.infer<typeof ServiceSchema>;
