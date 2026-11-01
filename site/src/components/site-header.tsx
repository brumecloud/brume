import Link from "next/link";

import { pages, site } from "~/content";

import { Brand } from "./brand";

export function SiteHeader() {
  return (
    <header className="absolute inset-x-0 top-0 z-10">
      <nav
        aria-label="Main"
        className="mx-auto flex w-full max-w-page items-center gap-6 px-6 py-5 text-[17px]"
      >
        <Brand />
        <ul className="ml-auto hidden items-center gap-7 text-sm text-ink-muted md:flex">
          {pages.map((page) => (
            <li key={page.path}>
              <Link href={page.path} className="transition-colors hover:text-ink">
                {page.title}
              </Link>
            </li>
          ))}
        </ul>
        <a
          href={site.repo}
          className="pixel-border ml-auto bg-surface/70 px-3 py-1.5 text-sm transition-colors hover:text-brand md:ml-0"
        >
          GitHub ↗
        </a>
      </nav>
    </header>
  );
}
