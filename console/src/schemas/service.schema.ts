import { z } from "zod";

export const ServiceSchema = z.object({
  name: z.string(),
  id: z.string(),
});

export type Service = z.infer<typeof ServiceSchema>;
