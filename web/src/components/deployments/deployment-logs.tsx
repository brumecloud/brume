import { useSuspenseQuery } from "@tanstack/react-query";
import { DownloadIcon, ScrollTextIcon, SearchIcon } from "lucide-react";
import { useState } from "react";

import { deploymentLogsQuery } from "~/api/deployments";
import type { DeploymentId, DeploymentLogStream } from "~/api/types";
import { LogRow } from "~/components/logs/log-row";
import { Button } from "~/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupInput } from "~/components/ui/input-group";
import { filterLogs, logsAsText } from "~/helpers/logs";

interface DeploymentLogsProps {
  readonly deploymentId: DeploymentId;
  readonly stream: DeploymentLogStream;
  readonly fileName: string;
}

// References: infra/screens/05b-deployment-build-logs-empty.png and 05c.
export function DeploymentLogs({ deploymentId, stream, fileName }: DeploymentLogsProps) {
  const { data: lines } = useSuspenseQuery(deploymentLogsQuery(deploymentId, stream));
  const [query, setQuery] = useState("");
  const [openLineId, setOpenLineId] = useState<string>();
  const visible = filterLogs(lines, { q: query });

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex gap-2 border-b px-6 py-3">
        <InputGroup className="flex-1">
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
          <InputGroupInput
            type="search"
            aria-label="Filtrer les logs"
            placeholder="Filtrer les logs…"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </InputGroup>
        <Button
          variant="outline"
          size="icon"
          aria-label="Télécharger les logs"
          disabled={lines.length === 0}
          onClick={() => {
            const url = URL.createObjectURL(new Blob([logsAsText(lines)], { type: "text/plain" }));
            const anchor = document.createElement("a");
            anchor.href = url;
            anchor.download = fileName;
            anchor.click();
            URL.revokeObjectURL(url);
          }}
        >
          <DownloadIcon />
        </Button>
      </div>
      <div className="flex-1 bg-terminal py-2">
        {visible.length === 0 ? (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <ScrollTextIcon />
              </EmptyMedia>
              <EmptyTitle>Aucun log</EmptyTitle>
              <EmptyDescription>
                Les logs apparaissent ici dès que le déploiement en produit.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          visible.map((line) => (
            <LogRow
              key={line.id}
              line={line}
              open={line.id === openLineId}
              onToggle={() => setOpenLineId(line.id === openLineId ? undefined : line.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}
