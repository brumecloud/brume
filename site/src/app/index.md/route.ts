import { landingMarkdown, markdownResponse } from "~/lib/markdown";

// Written to out/index.md by the static export.
export const dynamic = "force-static";

/** The page in Markdown, for agents and terminals: see src/lib/markdown.ts. */
export function GET(): Response {
  return markdownResponse(landingMarkdown());
}
