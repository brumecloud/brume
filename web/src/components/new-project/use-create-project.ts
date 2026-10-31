import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { createProject } from "~/api/projects";

export function useCreateProject() {
  const navigate = useNavigate();
  return useMutation({
    mutationFn: createProject,
    onSuccess: ({ projectId, environmentName }) =>
      navigate({ to: "/projects/$projectId/$env", params: { projectId, env: environmentName } }),
  });
}
