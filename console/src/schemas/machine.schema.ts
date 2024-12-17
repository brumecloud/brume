import { z } from "zod";

export const MachineSchema = z.object({
  id: z.string(),
  name: z.string(),
  ip: z.string(),
});

export type Machine = z.infer<typeof MachineSchema>;
