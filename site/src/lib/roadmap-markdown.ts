// The roadmap page as Markdown, for agents and terminals: /roadmap/index.md.

import { docsPage, milestones, site } from "~/content";
import { openDecisions, roadmapCopy, roadmapThemes, whenLabel } from "~/roadmap";

import { heading, list } from "./markdown";

export function roadmapMarkdown(): string {
  return [
    `# ${site.name}: ${roadmapCopy.title}`,
    `> ${roadmapCopy.lede}`,
    `${roadmapCopy.badge}. This is ${site.url}/roadmap in Markdown, for agents and terminals. Details: ${docsPage("how-it-works/roadmap")}`,

    heading(2, roadmapCopy.timelineTitle),
    ...milestones.flatMap(({ when, title, items }) => [
      heading(3, `${title} (${when})`),
      list(items),
    ]),

    heading(2, roadmapCopy.themesTitle),
    roadmapCopy.themesLede,
    ...roadmapThemes.flatMap((theme) => [
      heading(3, theme.title),
      list(
        theme.items.map(
          ({ title, brings, when }) =>
            `**${title}**${when ? ` (${whenLabel(when)})` : ""}: ${brings}`,
        ),
      ),
    ]),

    heading(2, roadmapCopy.openTitle),
    roadmapCopy.openLede,
    list(openDecisions.map(({ title, text }) => `**${title}:** ${text}`)),
  ]
    .join("\n\n")
    .concat("\n");
}
