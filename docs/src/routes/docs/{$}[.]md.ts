import { createFileRoute, notFound } from "@tanstack/react-router";
import { decodeMarkdownUrl } from "~/lib/shared";
import { docsLlms, source } from "~/lib/source";

/** Markdown twin of every page, for agents: `/docs/a/b.md`. */
export const Route = createFileRoute("/docs/{$}.md")({
  server: {
    handlers: {
      GET: async ({ params }) => {
        const slugs = decodeMarkdownUrl(params._splat?.split("/") ?? []);
        const page = source.getPage(slugs);
        if (!page) throw notFound();

        return new Response(await docsLlms.page(page), {
          headers: { "Content-Type": "text/markdown; charset=utf-8" },
        });
      },
    },
  },
});
