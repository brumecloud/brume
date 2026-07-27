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

function ThemeToggle() {
  return (
    <button
      aria-label="Toggle color theme"
      className="brume-theme-toggle"
      data-brume-theme-toggle
      type="button"
    >
      <span aria-hidden="true" className="brume-theme-icon" />
      <span className="brume-visually-hidden">Toggle color theme</span>
    </button>
  );
}

function NavigationLinks({
  navigation,
  currentRoute,
}: {
  navigation: NavigationItem[];
  currentRoute: string;
}) {
  return (
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
  );
}

function TableOfContents({ headings }: { headings: Heading[] }) {
  return (
    <ul>
      {headings.map((heading) => (
        <li className={`brume-toc-depth-${heading.depth}`} key={heading.id}>
          <a href={`#${heading.id}`}>{heading.title}</a>
        </li>
      ))}
    </ul>
  );
}

export function PageLayout({
  title,
  headerTitle,
  navigation,
  headings,
  currentRoute,
  children,
}: PropsWithChildren<{
  title: string;
  headerTitle?: string;
  navigation: NavigationItem[];
  headings: Heading[];
  currentRoute: string;
}>) {
  const hasSidebar = navigation.length > 1;
  const tocHeadings = headings.filter((heading) => heading.depth > 1);
  const hasToc = tocHeadings.length > 0;
  const className = [
    "brume-page",
    headerTitle ? "brume-has-header" : "",
    hasSidebar ? "brume-has-sidebar" : "",
    hasToc ? "brume-has-toc" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={className}>
      {headerTitle ? (
        <header className="brume-header">
          <a className="brume-header-title" href="__BRUME_BASE_PATH__">
            {headerTitle}
          </a>
          <ThemeToggle />
        </header>
      ) : null}
      <div className="brume-layout">
        {hasSidebar ? (
          <aside className="brume-sidebar">
            <strong className="brume-sidebar-title">{title}</strong>
            <nav aria-label="Plan pages">
              <NavigationLinks currentRoute={currentRoute} navigation={navigation} />
            </nav>
          </aside>
        ) : null}
        {hasSidebar || hasToc ? (
          <div className="brume-mobile-tools">
            {hasSidebar ? (
              <details className="brume-mobile-navigation">
                <summary>Pages</summary>
                <nav aria-label="Plan pages">
                  <NavigationLinks currentRoute={currentRoute} navigation={navigation} />
                </nav>
              </details>
            ) : null}
            {hasToc ? (
              <details className="brume-mobile-toc">
                <summary>On this page</summary>
                <nav aria-label="On this page">
                  <TableOfContents headings={tocHeadings} />
                </nav>
              </details>
            ) : null}
          </div>
        ) : null}
        <main className="brume-document">{children}</main>
        {hasToc ? (
          <nav aria-label="On this page" className="brume-toc">
            <strong>On this page</strong>
            <TableOfContents headings={tocHeadings} />
          </nav>
        ) : null}
      </div>
      <footer className="brume-footer">
        <span>
          Generated with <strong>Brume</strong>
        </span>
        {!headerTitle ? <ThemeToggle /> : null}
      </footer>
    </div>
  );
}
