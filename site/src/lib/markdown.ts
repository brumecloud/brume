// The landing page as Markdown, for agents and terminals: `curl https://brume.dev` gets it
// (vercel.json), and so does anyone who asks for /index.md. Built from the same copy as the page,
// in the same order.

import {
  brumeComparison,
  comparison,
  copy,
  designDoc,
  features,
  milestones,
  principles,
  requirements,
  site,
  steps,
  usualSetup,
  withBrume,
} from "~/content";

interface Item {
  readonly label: string;
  readonly title: string;
  readonly text: string;
}

const heading = (level: 2 | 3, title: string) => `${"#".repeat(level)} ${title}`;
const list = (items: readonly string[]) => items.map((item) => `- ${item}`).join("\n");
const cards = (items: readonly Item[]) =>
  items
    .map(({ label, title, text }) => `- **${title}** (${label.toLowerCase()}): ${text}`)
    .join("\n");

export function landingMarkdown(): string {
  const current = milestones.find((milestone) => milestone.status === "current");
  const rows = [...comparison, brumeComparison].map(
    ({ tool, panel, docker, attackerGets }) =>
      `| ${tool} | ${panel} | ${docker} | ${attackerGets} |`,
  );

  return [
    `# ${site.name}: ${copy.hero.title}`,
    `> ${copy.hero.lede}`,
    `This is ${site.url} in Markdown, for agents and terminals.${current ? ` Status: ${current.title} (${current.when}).` : ""} Source: ${site.repo}`,

    heading(2, "Install"),
    `\`\`\`sh\n${site.installCommand}\n\`\`\``,
    requirements,

    heading(2, copy.problem.title),
    copy.problem.lede,
    `The usual setup:\n\n${list(usualSetup)}`,
    `With ${site.name}:\n\n${list(withBrume)}`,

    heading(2, copy.howItWorks.title),
    steps
      .map(
        ({ title, text, command }, index) =>
          `${index + 1}. **${command ? `\`${title}\`` : title}**: ${text}`,
      )
      .join("\n"),

    heading(2, copy.console.title),
    copy.console.lede,
    `A live demo of the console, on demo data, runs in a browser at ${site.url}/#console.`,

    heading(2, copy.features.title),
    cards(features),

    heading(2, copy.security.title),
    copy.security.lede,
    `${copy.security.architecture} ${copy.security.architectureCaption}`,
    cards(principles),
    heading(3, copy.security.comparisonTitle),
    ["| Tool | Panel | Docker access | Attacker gets |", "| --- | --- | --- | --- |", ...rows].join(
      "\n",
    ),
    `${copy.security.comparisonSource} Details: ${designDoc("11-comparatif.md")}`,

    heading(2, copy.roadmap.title),
    ...milestones.flatMap(({ when, title, items }) => [
      heading(3, `${title} (${when})`),
      list(items),
    ]),

    heading(2, "Links"),
    list([
      `Source code: ${site.repo}`,
      `Issues: ${site.issues}`,
      `Install script: ${site.url}/install.sh`,
      `Design notes: ${site.repo}/tree/main/archi/base`,
    ]),
  ]
    .join("\n\n")
    .concat("\n");
}
