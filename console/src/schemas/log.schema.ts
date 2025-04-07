import { z } from "zod";

/**
 * @deprecated use direct validation from graphql
 */
export const LogSchema = z.object({
  id: z.string(),
  message: z.string(),
  level: z.union([z.literal("info"), z.literal("error")]),
  timestamp: z.string(),
  deploymentId: z.string(),
  serviceId: z.string(),
  deploymentName: z.string(),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Log = z.infer<typeof LogSchema>;
