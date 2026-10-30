import { Fragment } from "react";

import { SectionHeading } from "~/components/section";
import { Dim, Prompt, Terminal } from "~/components/terminal";
import { TextLink } from "~/components/text-link";
import { designDoc, designDocs, site, stack } from "~/content";

const nameWidth = Math.max(...designDocs.map((doc) => doc.file.length)) + 2;

export function OpenSource() {
  return (
    <section id="open-source" aria-labelledby="open-source-title" className="scroll-mt-8 py-24">
      <div className="mx-auto grid w-full max-w-page items-center gap-14 px-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)]">
        <div>
          <SectionHeading
            id="open-source-title"
            kicker="Open source"
            title="Designed in the open, before the first line of code."
            lede="Every decision is public: the threat model, the architecture, the data model, and the risks we accept for 0.1. Read it, question it, open an issue."
          />
          <ul aria-label="Stack" className="mt-8 flex flex-wrap gap-2.5">
            {stack.map((item) => (
              <li
                key={item}
                className="pixel-border bg-surface px-2.5 py-1 font-mono text-xs text-ink-muted"
              >
                {item}
              </li>
            ))}
          </ul>
          <p className="mt-8 flex flex-wrap gap-7 text-sm text-ink-muted">
            <TextLink href={site.designDocs}>Read the design docs ↗</TextLink>
            <TextLink href={site.repo}>Star on GitHub ↗</TextLink>
          </p>
        </div>
        <Terminal
          title="brume/archi/base"
          lines={[
            <Fragment key="ls">
              <Prompt />
              ls
            </Fragment>,
            ...designDocs.map((doc) => (
              <Fragment key={doc.file}>
                <a href={designDoc(doc.file)} className="hover:text-brand">
                  {doc.file}
                </a>
                {" ".repeat(nameWidth - doc.file.length)}
                <Dim># {doc.topic}</Dim>
              </Fragment>
            )),
            <Dim key="language"># written in French, for now</Dim>,
          ]}
        />
      </div>
    </section>
  );
}
