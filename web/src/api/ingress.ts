import { queryOptions } from "@tanstack/react-query";

import { parseIngress } from "~/helpers/ingress";

import { apiGet } from "./client";

// Null until `brume install --domain` sets up the public entry of the server.
export const ingressQuery = queryOptions({
  queryKey: ["ingress"],
  queryFn: () => apiGet("/api/ingress", parseIngress),
});
