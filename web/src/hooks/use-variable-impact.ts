import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useQuery } from "@tanstack/react-query";

import { allEnvironments } from "~/api/projects";
import { allServices } from "~/api/services";
import type { SharedVariableLevel } from "~/api/types";
import { variableImpactQuery } from "~/api/variables";
import { groupImpact } from "~/helpers/shared-variables";

// archi/base/13-modele-de-donnees.md#modifier-une-valeur-partagée: the services a change to a
// shared plain or build value redeploys, asked for when the caller mounts. `groups` feeds
// `ImpactList`; `count` is the server's, not the number of services the panel can name.
export function useVariableImpact(
  level: SharedVariableLevel,
  ownerId: string,
  name: string,
  enabled: boolean,
) {
  const impact = useQuery({ ...variableImpactQuery(level, ownerId, name), enabled });
  const { data: environments } = useLiveSuspenseQuery({ query: allEnvironments });
  const { data: services } = useLiveSuspenseQuery({ query: allServices });
  return {
    impact,
    groups: impact.data ? groupImpact(impact.data.services, environments, services) : [],
    count: impact.data?.services.length ?? 0,
  };
}
