import { queryOptions } from "@tanstack/react-query";

import { parseRepositories } from "~/helpers/new-project";

import { apiGet } from "./client";

export const repositoriesQuery = queryOptions({
  queryKey: ["github", "repositories"],
  queryFn: () => apiGet("/api/github/repositories", parseRepositories),
});
