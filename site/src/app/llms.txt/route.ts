import { llmsTxt } from "~/lib/llms";

// Written to out/llms.txt by the static export.
export const dynamic = "force-static";

/** The index agents look for first: every page, with its Markdown twin. */
export function GET(): Response {
  return new Response(llmsTxt(), { headers: { "Content-Type": "text/plain; charset=utf-8" } });
}
