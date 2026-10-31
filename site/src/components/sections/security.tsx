import Image from "next/image";

import logo from "~/assets/logo.png";
import { BrandIcon } from "~/components/brand-icon";
import { Card } from "~/components/card";
import { PinnedGrid } from "~/components/pinned-grid";
import { Section } from "~/components/section";
import { logoLit } from "~/components/tone";
import { brumeComparison, comparison, copy, designDoc, principles } from "~/content";

import { ArchitectureDiagram } from "./architecture-diagram";

export function Security() {
  return (
    <Section id="security" title={copy.security.title} lede={copy.security.lede} className="pt-8">
      <ArchitectureDiagram />

      <PinnedGrid
        count={principles.length}
        className="mt-10"
        gridClassName="grid gap-6 sm:grid-cols-2 lg:grid-cols-3"
      >
        {principles.map((principle) => (
          <Card key={principle.title} {...principle} />
        ))}
      </PinnedGrid>

      <h3 id="comparison-title" className="mt-20 text-xl font-semibold tracking-[-0.02em]">
        {copy.security.comparisonTitle}
      </h3>
      <div className="pixel-border reveal mt-5 overflow-x-auto bg-surface">
        <table
          aria-labelledby="comparison-title"
          className="w-full min-w-[640px] text-left text-sm"
        >
          <thead>
            <tr className="border-b border-edge">
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Tool
              </th>
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Panel
              </th>
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Docker access
              </th>
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Attacker gets
              </th>
            </tr>
          </thead>
          <tbody>
            {comparison.map((row) => (
              <tr key={row.tool} data-light className="group border-b border-edge text-ink-muted">
                <th scope="row" className="px-5 py-3.5 font-semibold text-ink">
                  <span className="flex items-center gap-2.5">
                    {row.icon ? (
                      <BrandIcon name={row.icon} className={`size-5 shrink-0 ${logoLit}`} />
                    ) : null}
                    {row.tool}
                  </span>
                </th>
                <td className="px-5 py-3.5">{row.panel}</td>
                <td className="px-5 py-3.5">{row.docker}</td>
                <td className="px-5 py-3.5">
                  <span
                    aria-hidden="true"
                    className="mr-1.5 text-ink-dim transition-colors duration-300 lit:text-rose-400"
                  >
                    ×
                  </span>
                  {row.attackerGets}
                </td>
              </tr>
            ))}
            <tr data-light className="group bg-highlight">
              <th scope="row" className="px-5 py-3.5 font-semibold">
                <span className="flex items-center gap-2.5">
                  <Image
                    src={logo}
                    alt=""
                    width={20}
                    height={20}
                    className={`shrink-0 rounded ${logoLit}`}
                  />
                  {brumeComparison.tool}
                </span>
              </th>
              <td className="px-5 py-3.5">{brumeComparison.panel}</td>
              <td className="px-5 py-3.5">{brumeComparison.docker}</td>
              <td className="px-5 py-3.5 transition-colors duration-300 lit:text-emerald-400">
                <span aria-hidden="true" className="mr-1.5">
                  ✓
                </span>
                {brumeComparison.attackerGets}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p className="mt-3.5 text-[13px] text-ink-dim">
        {copy.security.comparisonSource} Details in{" "}
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
