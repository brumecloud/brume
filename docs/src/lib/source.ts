import { llms, loader } from "fumadocs-core/source";
import { lucideIconsPlugin } from "fumadocs-core/source/lucide-icons";
import { pageSchema } from "fumadocs-core/source/schema";
import { defineDocs } from "fumadocs-mdx/macro";
import { z } from "zod";
import { docsRoute, getPageMarkdownUrl, siteUrl } from "./shared";

/**
 * Frontmatter of every page. See docs/AGENTS.md for what each field means.
 */
const brumePageSchema = pageSchema.extend({
  type: z.enum(["overview", "tutorial", "guide", "concept", "reference"]),
  status: z.enum(["planned", "available"]).default("planned"),
  sources: z.array(z.string()).default([]),
});

export const docs = defineDocs({
  dir: "content/docs",
  docs: {
    async: true,
    schema: brumePageSchema,
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
});

export const source = loader({
  source: docs.toFumadocsSource(),
  baseUrl: docsRoute,
  plugins: [lucideIconsPlugin()],
});

export const docsLlms = llms(source, {
  renderPage: async (page) => {
    const status =
      page.data.status === "planned"
        ? "\n> Status: planned. This page describes how Brume alpha 0.1 is designed to work. It is not built yet.\n"
        : "";

    return `# ${page.data.title}

URL: ${siteUrl}${page.url}
${page.data.description ? `\n${page.data.description}\n` : ""}${status}
${await page.data.getText("processed")}`;
  },
});

/**
 * `llms.txt`: the page index, with each link pointing to the Markdown twin of
 * the page so an agent gets Markdown on the first fetch.
 */
export function llmsIndex() {
  const index = docsLlms
    .index()
    .replace(/\]\((\/docs[^)]*)\)/g, (_, url: string) => {
      const slugs = url.slice(docsRoute.length).split("/").filter(Boolean);
      return `](${siteUrl}${getPageMarkdownUrl({ slugs }).url})`;
    });

  return `${llmsHeader}\n${index}`;
}

export const llmsHeader = `# Brume

> Brume is an open source, self-hostable alternative to Railway, Coolify and Dokploy. It runs on a single VPS, deploys containers and static sites, and keeps the admin panel off the public internet.

Brume is in its design phase: there is no release yet. Pages marked "planned" describe how alpha 0.1 is designed to work.

For AI agents: append \`.md\` to any docs URL to get the page as Markdown, for example ${siteUrl}${docsRoute}/how-it-works/architecture.md. The full documentation in one file is at ${siteUrl}/llms-full.txt.
`;
