import Image from "next/image";
import Link from "next/link";

import logo from "~/assets/logo.png";
import { BrandIcon } from "~/components/brand-icon";
import { HoverAnimate } from "~/components/hover-animate";
import { IconTile } from "~/components/icon-tile";
import { Section } from "~/components/section";
import { TextLink } from "~/components/text-link";
import { logoLit, toneTextLit } from "~/components/tone";
import { brumeComparison, comparison, copy, docsPage, securityClaims } from "~/content";

export function Security() {
  return (
    <Section id="security" title={copy.security.title} lede={copy.security.lede} className="pt-8">
      <ul className="mt-14 grid gap-12 md:grid-cols-3 md:gap-10">
        {securityClaims.map((claim) => (
          <li key={claim.title} className="reveal">
            <HoverAnimate light className="group">
              <div className="flex items-center gap-3">
                <IconTile icon={claim.icon} tone={claim.tone} />
                <p
                  className={`text-label text-ink-dim transition-colors duration-300 ${toneTextLit[claim.tone]}`}
                >
                  {claim.label}
                </p>
              </div>
              <h3 className="mt-5 text-[clamp(1.5rem,2.4vw,1.875rem)] leading-tight font-semibold tracking-[-0.03em]">
                {claim.title}
              </h3>
              <p className="mt-3 leading-relaxed text-pretty text-ink-muted">{claim.text}</p>
            </HoverAnimate>
          </li>
        ))}
      </ul>

      <h3 id="comparison-title" className="mt-20 text-xl font-semibold tracking-[-0.02em]">
        {copy.security.comparisonTitle}
      </h3>
      <div className="pixel-border reveal mt-5 overflow-x-auto bg-surface">
        <table
          aria-labelledby="comparison-title"
          className="w-full min-w-[520px] text-left text-sm"
        >
          <thead>
            <tr className="border-b border-edge">
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Tool
              </th>
              <th scope="col" className="text-label px-5 py-3.5 font-normal text-ink-dim">
                Admin panel
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
                  <Link
                    href={`/vs/${row.slug}`}
                    className="flex items-center gap-2.5 transition-colors hover:text-brand"
                  >
                    {row.icon ? (
                      <BrandIcon name={row.icon} className={`size-5 shrink-0 ${logoLit}`} />
                    ) : null}
                    {row.tool}
                  </Link>
                </th>
                <td className="px-5 py-3.5">{row.panel}</td>
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
          href={docsPage("how-it-works/comparison")}
          className="border-b border-dashed border-edge-strong text-ink-muted hover:text-brand"
        >
          our comparison
        </a>
        .
      </p>

      <p className="reveal mt-14 text-[17px] leading-relaxed text-pretty text-ink-muted">
        {copy.security.more} <TextLink href="/architecture">See how it&apos;s built →</TextLink>{" "}
        <span className="text-ink-dim">·</span>{" "}
        <TextLink href={docsPage("how-it-works/threat-model")}>Read the threat model ↗</TextLink>
      </p>
    </Section>
  );
}
