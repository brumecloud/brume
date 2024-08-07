import { z } from "zod";

export const LogSchema = z.object({
  message: z.string(),
  level: z.union([z.literal("info"), z.literal("error")]),
  timestamp: z.string(),
});

export type Log = z.infer<typeof LogSchema>;
