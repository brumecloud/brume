import { nextLogRetryDelay, parseLogLines } from "~/helpers/logs";

import { redirectToLogin } from "./client";
import { loginState } from "./session";
import type { EnvironmentId, LogLine } from "./types";

// Live logs of an environment, by SSE (archi/base/04-panel-admin.md). EventSource cannot send
// the X-Brume header: the server guards this GET with Fetch Metadata (Sec-Fetch-Site) instead,
// and an SSE stream has no side effect. Each message carries a batch of lines as JSON.
//
// The browser reconnects by itself after a dropped stream, but gives up for good when the server
// refuses the handshake (a 502 or 503 while telemetry or the engine restarts, a 401 once the
// session expires). The stream is then opened again with a backoff, unless the session is gone:
// that leaves for the login page, like the other API calls. Lines sent again after a reconnect
// keep their ids, and the explorer drops the duplicates.
export function subscribeToLogs(
  environmentId: EnvironmentId,
  onLines: (lines: LogLine[]) => void,
): () => void {
  let source: EventSource | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let delay: number | undefined;
  let stopped = false;

  const open = () => {
    const current = new EventSource(`/api/environments/${environmentId}/logs`);
    source = current;
    current.addEventListener("open", () => {
      delay = undefined;
    });
    current.addEventListener("message", (event: MessageEvent<string>) => {
      onLines(parseLogLines(JSON.parse(event.data)));
    });
    current.addEventListener("error", () => {
      if (current.readyState === EventSource.CLOSED && !stopped) {
        delay = nextLogRetryDelay(delay);
        timer = setTimeout(() => void reopen(), delay);
      }
    });
  };

  const reopen = async () => {
    const state = await loginState(null).catch(() => "unknown" as const);
    if (stopped) {
      return;
    }
    if (state === "signed-out") {
      redirectToLogin();
      return;
    }
    open();
  };

  open();
  return () => {
    stopped = true;
    clearTimeout(timer);
    source?.close();
  };
}
