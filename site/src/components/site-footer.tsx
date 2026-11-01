import Link from "next/link";
import type { ReactNode } from "react";

import { docsPage, site } from "~/content";
import { roadmapThemes, themeId } from "~/roadmap";
import { competitors } from "~/vs";
import type { Competitor } from "~/vs/model";

import { Brand } from "./brand";
import { BrandIcon } from "./brand-icon";
import type { BrandIconName } from "./brand-icons";
import { logoLit } from "./tone";

interface FooterLink {
  readonly href: string;
  readonly label: string;
  readonly icon?: BrandIconName;
}

const security: readonly FooterLink[] = [
  { href: "/#security", label: "Secure by design" },
  { href: "/architecture", label: "How Brume works" },
  { href: "/architecture#principles", label: "The ideas behind it" },
  { href: "/architecture#components", label: "Every component" },
  { href: docsPage("how-it-works/threat-model"), label: "Threat model ↗" },
];

const roadmap: readonly FooterLink[] = [
  { href: "/roadmap", label: "At a glance" },
  ...roadmapThemes.map((theme) => ({
    href: `/roadmap#${themeId(theme.title)}`,
    label: theme.title,
  })),
  { href: "/roadmap#open", label: "Still open" },
];

const links: readonly FooterLink[] = [
  { href: site.repo, label: "GitHub ↗" },
  { href: `${site.repo}/releases`, label: "Releases ↗" },
  { href: `${site.repo}/tree/main/docs/content/docs`, label: "Docs ↗" },
  { href: site.issues, label: "Issues ↗" },
  { href: "/llms.txt", label: "llms.txt, for agents" },
];

const comparisons = (kind: Competitor["kind"]): readonly FooterLink[] =>
  competitors
    .filter((competitor) => competitor.kind === kind)
    .map((competitor) => ({
      href: `/vs/${competitor.slug}`,
      label: `vs ${competitor.name}`,
      icon: competitor.icon,
    }));

function LinkList({ items }: { readonly items: readonly FooterLink[] }) {
  return (
    <ul className="mt-4 grid content-start gap-2.5 text-ink-muted">
      {items.map((item) => (
        <li key={item.href}>
          {/* Pages go through Link; text files and external sites are plain links. */}
          {item.href.startsWith("/") && !/\.(md|txt)$/.test(item.href) ? (
            <Link
              href={item.href}
              className="group inline-flex items-center gap-2 transition-colors hover:text-brand"
            >
              {item.icon ? (
                <BrandIcon name={item.icon} className={`size-4 shrink-0 ${logoLit}`} />
              ) : null}
              {item.label}
            </Link>
          ) : (
            <a href={item.href} className="transition-colors hover:text-brand">
              {item.label}
            </a>
          )}
        </li>
      ))}
    </ul>
  );
}

function Column({ title, children }: { readonly title: string; readonly children: ReactNode }) {
  return (
    <div>
      <h2 className="text-label text-ink-dim">{title}</h2>
      {children}
    </div>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t-2 border-edge text-sm">
      <nav
        aria-label="Footer"
        className="mx-auto grid w-full max-w-page gap-x-8 gap-y-10 px-6 py-14 sm:grid-cols-2 lg:grid-cols-[1.3fr_1fr_1fr_1.6fr_1fr]"
      >
        <div className="sm:col-span-2 lg:col-span-1">
          <Brand size={24} className="text-[17px]" />
          <p className="mt-4 max-w-[16rem] leading-relaxed text-ink-muted">
            Deploy anything on your own server.
          </p>
          <p className="mt-3 text-ink-dim">Alpha 0.1 · Open source</p>
        </div>
        <Column title="Security">
          <LinkList items={security} />
        </Column>
        <Column title="Roadmap">
          <LinkList items={roadmap} />
        </Column>
        <Column title="Compare">
          <div className="grid grid-cols-2 items-start gap-x-6">
            <LinkList items={comparisons("self-hosted")} />
            <LinkList items={comparisons("managed")} />
          </div>
        </Column>
        <Column title="Links">
          <LinkList items={links} />
        </Column>
      </nav>
    </footer>
  );
}
