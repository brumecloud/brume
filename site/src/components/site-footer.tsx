import { site } from "~/content";

import { Brand } from "./brand";

const links = [
  { href: site.repo, label: "GitHub" },
  { href: "#security", label: "Security" },
  { href: site.issues, label: "Issues" },
];

export function SiteFooter() {
  return (
    <footer className="border-t-2 border-edge">
      <div className="mx-auto flex w-full max-w-page flex-wrap items-center gap-x-8 gap-y-3 px-6 py-5 text-sm">
        <Brand size={20} className="text-[15px]" />
        <nav aria-label="Footer">
          <ul className="flex flex-wrap gap-6 text-ink-muted">
            {links.map((link) => (
              <li key={link.label}>
                <a href={link.href} className="transition-colors hover:text-brand">
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
        <p className="text-ink-dim sm:ml-auto">Alpha 0.1 · Open source</p>
      </div>
    </footer>
  );
}
