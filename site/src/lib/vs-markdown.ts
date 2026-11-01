// The comparison pages as Markdown, for agents and terminals: /vs/index.md and
// /vs/<slug>/index.md. Built from the same data as the pages, in the same order.

import {
  checkedOn,
  competitorsOfKind,
  otherComparisons,
  vsCopy,
  vsGroups,
} from "~/components/vs/shared";
import { requirements, site } from "~/content";
import { competitor } from "~/vs";
import { brumeRows } from "~/vs/brume";
import type { VsSlug } from "~/vs/model";

import { heading, list } from "./markdown";

const pageUrl = (slug: VsSlug) => `${site.url}/vs/${slug}`;
const pageLink = (name: string, slug: VsSlug) => `[${vsCopy.title(name)}](${pageUrl(slug)})`;
// A table cell holds one line, and a pipe would end it.
const cell = (text: string) => text.replaceAll("\n", " ").replaceAll("|", "\\|");
const tableRow = (cells: readonly string[]) => `| ${cells.join(" | ")} |`;

export function vsMarkdown(slug: VsSlug): string {
  const compared = competitor(slug);
  const table = [
    tableRow(["", site.name, cell(compared.name)]),
    tableRow(["---", "---", "---"]),
    ...compared.rows.map(({ key, them, source }) =>
      tableRow([
        cell(brumeRows[key].label),
        cell(brumeRows[key].brume),
        source ? `[${cell(them)}](${source})` : cell(them),
      ]),
    ),
  ].join("\n");

  return [
    `# ${vsCopy.title(compared.name)}`,
    `> ${compared.verdict}`,
    `${vsCopy.badge}. This is ${pageUrl(slug)} in Markdown, for agents and terminals.`,
    table,
    vsCopy.checked(checkedOn(compared.checked)),

    heading(2, vsCopy.different),
    list(compared.different),
    heading(2, vsCopy.better(compared.name)),
    list(compared.better),
    `**${vsCopy.pickThem(compared.name)}** ${compared.pickThem}`,
    `**${vsCopy.pickBrume}** ${compared.pickBrume}`,

    heading(2, "Install"),
    `\`\`\`sh\n${site.installCommand}\n\`\`\``,
    requirements,

    heading(2, vsCopy.others),
    list(otherComparisons(compared).map((other) => pageLink(other.name, other.slug))),
  ]
    .filter(Boolean)
    .join("\n\n")
    .concat("\n");
}

export function vsIndexMarkdown(): string {
  return [
    `# ${vsCopy.index.title}`,
    `> ${vsCopy.index.lede}`,
    `${vsCopy.badge}. This is ${site.url}/vs in Markdown, for agents and terminals.`,
    ...vsGroups.flatMap(({ kind, title }) => [
      heading(2, title),
      list(
        competitorsOfKind(kind).map(
          (other) => `${pageLink(other.name, other.slug)}: ${other.oneLiner}`,
        ),
      ),
    ]),
  ]
    .join("\n\n")
    .concat("\n");
}
