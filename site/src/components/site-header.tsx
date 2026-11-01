import Link from "next/link";

import { pages, site } from "~/content";

import { Brand } from "./brand";

export function SiteHeader() {
  return (
    <header className="absolute inset-x-0 top-0 z-10">
      <nav
        aria-label="Main"
        className="mx-auto flex w-full max-w-page flex-wrap items-center gap-x-6 gap-y-3 px-6 py-5 text-[17px]"
      >
        <Brand />
        {/* On a phone, the links take their own row under the logo. */}
        <ul className="order-last flex w-full items-center gap-6 text-sm text-ink-muted md:order-none md:ml-auto md:w-auto md:gap-7">
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
