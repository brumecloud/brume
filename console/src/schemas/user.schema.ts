import { z } from "zod";

/**
 * @deprecated use direct validation from graphql
 */
export const UserSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  avatar: z.string().url(),
});

/**
 * @deprecated use type from @/_apollo/graphql
 */
export type User = z.infer<typeof UserSchema>;
