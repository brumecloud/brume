import type { RenderFailure, RenderRequest, RenderResponse } from "./protocol";
import { renderSite } from "./render-site";

async function readStandardInput(): Promise<string> {
  return Bun.stdin.text();
}

function failure(error: unknown): RenderFailure {
  const candidate = error as Error & {
    file?: string;
    line?: number;
    column?: number;
    position?: { start?: { line?: number; column?: number } };
  };
  return {
    ok: false,
    message: candidate.message || String(error),
    file: candidate.file,
    line: candidate.line ?? candidate.position?.start?.line,
    column: candidate.column ?? candidate.position?.start?.column,
  };
}

let response: RenderResponse;
try {
  const request = JSON.parse(await readStandardInput()) as RenderRequest;
  response = await renderSite(request);
} catch (error) {
  response = failure(error);
}

process.stdout.write(`${JSON.stringify(response)}\n`);
if (!response.ok) process.exitCode = 1;
