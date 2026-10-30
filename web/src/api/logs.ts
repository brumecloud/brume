import { parseLogLines } from "~/helpers/logs";

import type { EnvironmentId, LogLine } from "./types";

// Live logs of an environment, by SSE (archi/base/04-panel-admin.md). EventSource cannot send
// the X-Brume header: the server guards this GET with Fetch Metadata (Sec-Fetch-Site) instead,
// and an SSE stream has no side effect. Each message carries a batch of lines as JSON.
export function subscribeToLogs(
  environmentId: EnvironmentId,
  onLines: (lines: LogLine[]) => void,
): () => void {
  const source = new EventSource(`/api/environments/${environmentId}/logs`);
  source.addEventListener("message", (event: MessageEvent<string>) => {
    onLines(parseLogLines(JSON.parse(event.data)));
  });
  return () => source.close();
}
