import { useSuspenseQuery } from "@tanstack/react-query";

import { deploymentLogsQuery } from "~/api/deployments";
import type { DeploymentId, DeploymentLogStream } from "~/api/types";
import { LogRow } from "~/components/logs/log-row";

interface DeploymentLogsProps {
  readonly deploymentId: DeploymentId;
  readonly stream: DeploymentLogStream;
}

// References: infra/screens/05b-deployment-build-logs-empty.png and 05c.
export function DeploymentLogs({ deploymentId, stream }: DeploymentLogsProps) {
  const { data: lines } = useSuspenseQuery(deploymentLogsQuery(deploymentId, stream));

  return (
    <div className="rounded-xl border bg-background py-2">
      {lines.map((line) => (
        <LogRow key={line.id} line={line} />
      ))}
    </div>
  );
}
