import { z } from "zod";

export const ProjectSchema = z.object({
  name: z.string(),
  description: z.string(),
  id: z.string(),
});

export type Project = z.infer<typeof ProjectSchema>;
