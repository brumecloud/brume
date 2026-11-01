import { llmsFullTxt } from "~/lib/llms";

// Written to out/llms-full.txt by the static export.
export const dynamic = "force-static";

/** Every page of the site in Markdown, in one file. */
export function GET(): Response {
  return new Response(llmsFullTxt(), { headers: { "Content-Type": "text/plain; charset=utf-8" } });
}
