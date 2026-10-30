import { site } from "~/content";

import { Brand } from "./brand";

const links = [
  { href: site.repo, label: "github" },
  { href: site.designDocs, label: "design" },
  { href: "#security", label: "security" },
  { href: site.issues, label: "issues" },
];

/** Styled like a terminal multiplexer status bar. */
export function SiteFooter() {
  return (
    <footer className="border-t-2 border-edge">
      <div className="mx-auto flex w-full max-w-page flex-wrap items-center gap-x-6 gap-y-3 px-6 py-4 text-sm">
        <Brand size={20} className="text-[15px]" />
        <nav aria-label="Footer">
          <ul className="flex flex-wrap gap-3.5 font-mono text-[12.5px] text-ink-muted">
            {links.map((link, index) => (
              <li key={link.label}>
                <a href={link.href} className="transition-colors hover:text-brand">
                  [{index}:{link.label}]
                </a>
              </li>
            ))}
          </ul>
        </nav>
        <p className="font-mono text-[12.5px] text-ink-dim sm:ml-auto">alpha 0.1 · open source</p>
      </div>
    </footer>
  );
}
