import { z } from "zod";

export const ServiceSchema = z.object({
  __typename: z.literal("Service"),
  name: z.string(),
  id: z.string(),
});

export type Service = z.infer<typeof ServiceSchema>;
