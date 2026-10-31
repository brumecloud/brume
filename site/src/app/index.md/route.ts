import { landingMarkdown } from "~/lib/markdown";

// Written to out/index.md by the static export.
export const dynamic = "force-static";

/** The page in Markdown, for agents and terminals: see src/lib/markdown.ts. */
export function GET(): Response {
  return new Response(landingMarkdown(), {
    headers: { "Content-Type": "text/markdown; charset=utf-8" },
  });
}
