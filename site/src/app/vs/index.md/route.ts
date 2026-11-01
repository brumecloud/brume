import { markdownResponse } from "~/lib/markdown";
import { vsIndexMarkdown } from "~/lib/vs-markdown";

// Written to out/vs/index.md by the static export.
export const dynamic = "force-static";

/** The list of comparisons in Markdown, for agents and terminals. */
export function GET(): Response {
  return markdownResponse(vsIndexMarkdown());
}
