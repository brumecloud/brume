import { z } from "zod";

export const UserSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  avatar: z.string().url(),
});

export type User = z.infer<typeof UserSchema>;
