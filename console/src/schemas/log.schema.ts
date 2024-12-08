import { z } from "zod";

export const LogSchema = z.object({
  id: z.string(),
  message: z.string(),
  level: z.union([z.literal("info"), z.literal("error")]),
  timestamp: z.string(),
  deploymentId: z.string(),
  deploymentName: z.string(),
});

export type Log = z.infer<typeof LogSchema>;
