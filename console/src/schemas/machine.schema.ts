import { z } from "zod";

/**
 * @deprecated use direct validation from graphql
 */
export const MachineSchema = z.object({
  id: z.string(),
  name: z.string(),
  ip: z.string(),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type Machine = z.infer<typeof MachineSchema>;
