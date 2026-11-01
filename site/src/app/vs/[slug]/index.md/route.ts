import { findCompetitor } from "~/components/vs/shared";
import { markdownResponse } from "~/lib/markdown";
import { vsMarkdown } from "~/lib/vs-markdown";
import { competitors } from "~/vs";

// Written to out/vs/<slug>/index.md by the static export, one per comparison and no other.
export const dynamic = "force-static";
export const dynamicParams = false;

export function generateStaticParams(): { slug: string }[] {
  return competitors.map(({ slug }) => ({ slug }));
}

/** A comparison page in Markdown, for agents and terminals. */
export async function GET(
  _request: Request,
  { params }: { params: Promise<{ slug: string }> },
): Promise<Response> {
  const found = findCompetitor((await params).slug);
  if (!found) return new Response("Not found", { status: 404 });
  return markdownResponse(vsMarkdown(found.slug));
}
