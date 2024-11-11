import { z } from "zod";

export const DockerRunnerSchema = z.object({
  type: z.literal("generic-docker"),
  data: z.object({
    healthCheckURL: z.string(),
    command: z.string().optional(),
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
  builder: BuilderSchema,
  runner: RunnerSchema,
});

export type Service = z.infer<typeof ServiceSchema>;
