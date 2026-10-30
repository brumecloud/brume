import { queryOptions } from "@tanstack/react-query";

import { parseBranches, repositoryPath } from "~/helpers/github";
import { parseRepositories } from "~/helpers/new-project";

import { apiGet } from "./client";

export const repositoriesQuery = queryOptions({
  queryKey: ["github", "repositories"],
  queryFn: () => apiGet("/api/github/repositories", parseRepositories),
});

export const branchesQuery = (repository: string) =>
  queryOptions({
    queryKey: ["github", "branches", repository],
    queryFn: () => {
      const path = repositoryPath(repository);
      if (!path) {
        throw new Error("Dépôt GitHub invalide.");
      }
      return apiGet(`/api/github/repositories/${path}/branches`, parseBranches);
    },
  });
