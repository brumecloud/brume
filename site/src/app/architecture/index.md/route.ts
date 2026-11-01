import { architectureMarkdown } from "~/lib/architecture-markdown";
import { markdownResponse } from "~/lib/markdown";

// Written to out/architecture/index.md by the static export.
export const dynamic = "force-static";

/** The architecture page in Markdown, for agents and terminals. */
export function GET(): Response {
  return markdownResponse(architectureMarkdown());
}
