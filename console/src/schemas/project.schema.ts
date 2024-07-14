import { z } from "zod";

import { ServiceSchema } from "./service.schema";

export const ProjectSchema = z.object({
  name: z.string(),
  description: z.string(),
  id: z.string(),
  services: z.array(ServiceSchema),
});

export type Project = z.infer<typeof ProjectSchema>;
