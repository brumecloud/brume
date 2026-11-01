// The architecture page as Markdown, for agents and terminals: /architecture/index.md.

import { archComponents, componentById } from "~/architecture/components";
import { architectureCopy } from "~/architecture/copy";
import { neighbors } from "~/architecture/model";
import { tours } from "~/architecture/tours";
import { docsPage, site } from "~/content";

import { heading, list } from "./markdown";

export function architectureMarkdown(): string {
  return [
    `# ${architectureCopy.title.replace(/\.$/, "")}`,
    `> ${architectureCopy.lede}`,
    `${architectureCopy.badge}. ${architectureCopy.modes} This is ${site.url}/architecture in Markdown, for agents and terminals. The page itself has an interactive diagram.`,

    heading(2, architectureCopy.principlesTitle),
    list(architectureCopy.principles),

    heading(2, architectureCopy.componentsTitle),
    ...archComponents.flatMap((component) =>
      [
        heading(3, `${component.label} (${component.detail})`),
        component.why,
        `How it works:\n\n${list(component.how)}`,
        `Security rules:\n\n${list(component.rules.map(({ rule, why }) => `**${rule}** ${why}`))}`,
        component.ifCompromised ? `If it's compromised: ${component.ifCompromised}` : "",
        `Talks to: ${neighbors(component.id)
          .map((id) => componentById(id).label)
          .join(", ")}. Docs: ${docsPage(component.docs)}`,
      ].filter(Boolean),
    ),

    heading(2, "Tours"),
    ...tours.flatMap((tour) => [
      heading(3, tour.title),
      tour.steps.map((step, index) => `${index + 1}. ${step.caption}`).join("\n"),
    ]),
  ]
    .join("\n\n")
    .concat("\n");
}
