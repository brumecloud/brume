// /llms.txt and /llms-full.txt, the entry points agents look for (https://llmstxt.org): an index
// of every page with its Markdown twin, and every twin in one file.

import { pages, site } from "~/content";
import { competitors } from "~/vs";

import { architectureMarkdown } from "./architecture-markdown";
import { landingMarkdown } from "./markdown";
import { roadmapMarkdown } from "./roadmap-markdown";
import { vsIndexMarkdown, vsMarkdown } from "./vs-markdown";

/** A page's Markdown twin, at `<path>/index.md`. */
const twin = (path: string) => `${site.url}${path}/index.md`;

export function llmsTxt(): string {
  return [
    `# ${site.name}`,
    `> ${site.description}`,
    `Every page of ${site.url} also exists in Markdown, at its path followed by \`/index.md\`. A request that asks for \`text/markdown\` gets it at the page's own URL. The whole site in one file: ${site.url}/llms-full.txt`,
    "## Pages",
    [
      `- [${site.name}: just push it](${twin("")}): ${site.description}`,
      ...pages.map((page) => `- [${page.title}](${twin(page.path)}): ${page.description}`),
    ].join("\n"),
    "## Comparisons",
    competitors
      .map(
        (competitor) =>
          `- [Brume vs ${competitor.name}](${twin(`/vs/${competitor.slug}`)}): ${competitor.description}`,
      )
      .join("\n"),
    "## Optional",
    [
      `- [Source code](${site.repo}): the repository, with the design notes in archi/base/`,
      `- [Docs](${site.repo}/tree/main/docs/content/docs): how Brume works, in depth`,
      `- [Install script](${site.url}/install.sh): ${site.installCommand}`,
    ].join("\n"),
  ]
    .join("\n\n")
    .concat("\n");
}

export function llmsFullTxt(): string {
  return [
    landingMarkdown(),
    architectureMarkdown(),
    roadmapMarkdown(),
    vsIndexMarkdown(),
    ...competitors.map((competitor) => vsMarkdown(competitor.slug)),
  ].join("\n---\n\n");
}
