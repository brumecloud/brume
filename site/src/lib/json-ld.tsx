// Structured data for search engines: what each page is, and where it sits in the site.

import { site } from "~/content";

/** Brume itself, as search engines describe software. The landing page is about it. */
export const brumeSoftware = {
  "@type": "SoftwareApplication",
  name: site.name,
  url: site.url,
  description: site.description,
  applicationCategory: "DeveloperApplication",
  operatingSystem: "Linux",
  isAccessibleForFree: true,
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  sameAs: [site.repo],
} as const;

interface Crumb {
  readonly name: string;
  readonly path: string;
}

interface PageJsonLdInput {
  readonly path: string;
  readonly name: string;
  readonly description: string;
  /** The pages above this one, root excluded: [{ name: "Compare", path: "/vs" }]. */
  readonly parents?: readonly Crumb[];
  /** Extra WebPage fields, such as `mentions`. */
  readonly extra?: Record<string, unknown>;
}

/** A sub-page: a WebPage about Brume, and its breadcrumb from the root. */
export function pageJsonLd({ path, name, description, parents = [], extra }: PageJsonLdInput) {
  const url = `${site.url}${path}`;
  const crumbs: readonly Crumb[] = [{ name: site.name, path: "" }, ...parents, { name, path }];
  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "WebPage",
        "@id": url,
        url,
        name,
        description,
        inLanguage: "en",
        isPartOf: { "@type": "WebSite", name: site.name, url: site.url },
        about: brumeSoftware,
        ...extra,
      },
      {
        "@type": "BreadcrumbList",
        itemListElement: crumbs.map((crumb, index) => ({
          "@type": "ListItem",
          position: index + 1,
          name: crumb.name,
          item: `${site.url}${crumb.path}`,
        })),
      },
    ],
  };
}

/** Renders structured data, with "<" escaped so the JSON cannot close the script. */
export function JsonLd({ data }: { readonly data: object }) {
  return (
    <script
      type="application/ld+json"
      // oxlint-disable-next-line react/no-danger -- static JSON-LD, with "<" escaped so it cannot close the script
      dangerouslySetInnerHTML={{ __html: JSON.stringify(data).replaceAll("<", "\\u003c") }}
    />
  );
}
