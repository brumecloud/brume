// The landing page as Markdown, for agents and terminals: `curl https://brume.dev` gets it
// (vercel.json), and so does anyone who asks for /index.md. Built from the same copy as the page,
// in the same order.

import {
  agentPoints,
  brumeComparison,
  comparison,
  copy,
  deployFeatures,
  docsPage,
  heroPoints,
  milestones,
  requirements,
  securityClaims,
  setupFeatures,
  site,
  steps,
} from "~/content";

interface Item {
  readonly label: string;
  readonly title: string;
  readonly text: string;
}

/** A Markdown twin, as the static export writes it: the HTML pages link to these. */
export function markdownResponse(body: string): Response {
  return new Response(body, { headers: { "Content-Type": "text/markdown; charset=utf-8" } });
}

export const heading = (level: 2 | 3, title: string) => `${"#".repeat(level)} ${title}`;
export const list = (items: readonly string[]) => items.map((item) => `- ${item}`).join("\n");
const cards = (items: readonly Item[]) =>
  items
    .map(({ label, title, text }) => `- **${title}** (${label.toLowerCase()}): ${text}`)
    .join("\n");

export function landingMarkdown(): string {
  const current = milestones.find((milestone) => milestone.status === "current");
  const rows = [...comparison, brumeComparison].map(
    ({ tool, panel, attackerGets }) => `| ${tool} | ${panel} | ${attackerGets} |`,
  );

  return [
    `# ${site.name}: ${copy.hero.title}`,
    `> ${copy.hero.lede}`,
    heroPoints.join(" · "),
    `This is ${site.url} in Markdown, for agents and terminals.${current ? ` Status: ${current.title} (${current.when}).` : ""} Source: ${site.repo}`,

    heading(2, "Install"),
    `\`\`\`sh\n${site.installCommand}\n\`\`\``,
    requirements,

    heading(2, copy.deploy.title),
    copy.deploy.lede,
    `${copy.deploy.stack}.`,
    cards(deployFeatures),

    heading(2, copy.security.title),
    copy.security.lede,
    cards(securityClaims),
    heading(3, copy.security.comparisonTitle),
    ["| Tool | Admin panel | Attacker gets |", "| --- | --- | --- |", ...rows].join("\n"),
    `${copy.security.comparisonSource} Details: ${docsPage("how-it-works/comparison")}`,
    `${copy.security.more} Threat model: ${docsPage("how-it-works/threat-model")}`,

    heading(2, copy.console.title),
    copy.console.lede,
    `A live demo of the console, on demo data, runs in a browser at ${site.url}/#console.`,
    heading(3, copy.console.agentTitle),
    copy.console.agentLede,
    list(agentPoints),

    heading(2, copy.setup.title),
    copy.setup.lede,
    steps
      .map(
        ({ title, text, command }, index) =>
          `${index + 1}. **${command ? `\`${title}\`` : title}**: ${text}`,
      )
      .join("\n"),
    cards(setupFeatures),

    heading(2, copy.roadmap.title),
    ...milestones.flatMap(({ when, title, items }) => [
      heading(3, `${title} (${when})`),
      list(items),
    ]),

    heading(2, copy.closing.title),
    copy.closing.lede,

    heading(2, "Links"),
    list([
      `Source code: ${site.repo}`,
      `Issues: ${site.issues}`,
      `Install script: ${site.url}/install.sh`,
      `Docs: ${site.repo}/tree/main/docs/content/docs`,
      `Design notes: ${site.repo}/tree/main/archi/base`,
    ]),
  ]
    .join("\n\n")
    .concat("\n");
}
