import { markdownResponse } from "~/lib/markdown";
import { roadmapMarkdown } from "~/lib/roadmap-markdown";

// Written to out/roadmap/index.md by the static export.
export const dynamic = "force-static";

/** The roadmap page in Markdown, for agents and terminals. */
export function GET(): Response {
  return markdownResponse(roadmapMarkdown());
}
