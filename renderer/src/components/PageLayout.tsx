import type { PropsWithChildren } from "react";

export type NavigationItem = {
  route: string;
  title: string;
};

export type Heading = {
  depth: number;
  id: string;
  title: string;
};

export function PageLayout({
  title,
  navigation,
  headings,
  currentRoute,
  children,
}: PropsWithChildren<{
  title: string;
  navigation: NavigationItem[];
  headings: Heading[];
  currentRoute: string;
}>) {
  return (
    <div className="brume-layout">
      {navigation.length > 1 ? (
        <nav aria-label="Plan pages" className="brume-sidebar">
          <strong className="brume-sidebar-title">{title}</strong>
          <ul>
            {navigation.map((item) => (
              <li key={item.route}>
                <a
                  aria-current={item.route === currentRoute ? "page" : undefined}
                  href={`__BRUME_BASE_PATH__${item.route}`}
                >
                  {item.title}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      ) : null}
      <main className="brume-document">{children}</main>
      {headings.length > 1 ? (
        <nav aria-label="On this page" className="brume-toc">
          <strong>On this page</strong>
          <ul>
            {headings.map((heading) => (
              <li className={`brume-toc-depth-${heading.depth}`} key={heading.id}>
                <a href={`#${heading.id}`}>{heading.title}</a>
              </li>
            ))}
          </ul>
        </nav>
      ) : null}
    </div>
  );
}

