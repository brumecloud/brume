import { Card } from "~/components/card";
import { Section } from "~/components/section";
import { brumeComparison, comparison, designDoc, principles } from "~/content";

import { ArchitectureDiagram } from "./architecture-diagram";

export function Security() {
  return (
    <Section
      id="security"
      kicker="Security"
      title="Built as if every part of it will be attacked."
      lede="Brume is one signed binary, split into several services. Each runs as its own system user, with its own permissions. A flaw in one of them doesn't hand over the others."
      className="pt-8"
    >
      <ArchitectureDiagram />

      <div className="mt-10 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {principles.map((principle) => (
          <Card key={principle.title} {...principle} />
        ))}
      </div>

      <h3 id="comparison-title" className="mt-20 text-xl font-semibold tracking-[-0.02em]">
        If the admin panel is compromised…
      </h3>
      <div className="pixel-border mt-5 overflow-x-auto bg-surface">
        <table
          aria-labelledby="comparison-title"
          className="w-full min-w-[640px] text-left text-sm"
        >
          <thead>
            <tr className="border-b border-edge">
              <th scope="col" className="text-pixel px-5 py-3.5 font-normal text-ink-dim">
                Tool
              </th>
              <th scope="col" className="text-pixel px-5 py-3.5 font-normal text-ink-dim">
                Panel
              </th>
              <th scope="col" className="text-pixel px-5 py-3.5 font-normal text-ink-dim">
                Docker access
              </th>
              <th scope="col" className="text-pixel px-5 py-3.5 font-normal text-ink-dim">
                Attacker gets
              </th>
            </tr>
          </thead>
          <tbody>
            {comparison.map((row) => (
              <tr key={row.tool} className="border-b border-edge text-ink-muted">
                <th scope="row" className="px-5 py-3.5 font-semibold text-ink">
                  {row.tool}
                </th>
                <td className="px-5 py-3.5">{row.panel}</td>
                <td className="px-5 py-3.5">{row.docker}</td>
                <td className="px-5 py-3.5">
                  <span aria-hidden="true" className="mr-1.5 font-mono text-ink-dim">
                    ×
                  </span>
                  {row.attackerGets}
                </td>
              </tr>
            ))}
            <tr className="bg-highlight">
              <th scope="row" className="px-5 py-3.5 font-semibold">
                {brumeComparison.tool}
              </th>
              <td className="px-5 py-3.5">{brumeComparison.panel}</td>
              <td className="px-5 py-3.5">{brumeComparison.docker}</td>
              <td className="px-5 py-3.5 text-brand">
                <span aria-hidden="true" className="mr-1.5 font-mono">
                  ✓
                </span>
                {brumeComparison.attackerGets}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p className="mt-3.5 text-[13px] text-ink-dim">
        Based on each project&apos;s installation docs. Details in{" "}
        <a
          href={designDoc("11-comparatif.md")}
          className="border-b border-dashed border-edge-strong text-ink-muted hover:text-brand"
        >
          our comparison
        </a>
        .
      </p>
    </Section>
  );
}
