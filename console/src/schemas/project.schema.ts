import { z } from "zod";

import { ServiceSchema } from "./service.schema";

/**
 * @deprecated use direct validation from graphql
 */
export const ProjectSchema = z.object({
  __typename: z.literal("Project"),
  name: z.string(),
  description: z.string(),
  id: z.string(),
  services: z.array(ServiceSchema),
  isDirty: z.boolean(),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Project = z.infer<typeof ProjectSchema>;
