import type { Metadata } from "next";

import { site } from "~/content";

// The share images of the site root (src/app/opengraph-image.png and twitter-image.png). A page
// that sets its own `openGraph` or `twitter` replaces the root's objects whole, images included.
const shareImage = {
  width: 1200,
  height: 630,
  alt: "Brume: just push it. Open source deploys on your own server, under a pixel night sky.",
} as const;

interface PageMetadataInput {
  /** The page's path, such as "/architecture". Its Markdown twin is at `<path>/index.md`. */
  readonly path: string;
  /**
   * The page's name: the <title> adds " · Brume" after it. `{ absolute }` is the whole <title>,
   * for a title that already names Brume, such as a comparison page's.
   */
  readonly title: string | { readonly absolute: string };
  readonly description: string;
}

/** The metadata of a sub-page: its title, canonical URL, Markdown twin and share cards. */
export function pageMetadata({ path, title, description }: PageMetadataInput): Metadata {
  const name = typeof title === "string" ? title : title.absolute;
  return {
    title: typeof title === "string" ? `${title} · ${site.name}` : title.absolute,
    description,
    alternates: { canonical: path, types: { "text/markdown": `${path}/index.md` } },
    openGraph: {
      type: "website",
      url: path,
      siteName: site.name,
      locale: "en_US",
      title: name,
      description,
      images: [{ url: "/opengraph-image.png", ...shareImage }],
    },
    twitter: {
      card: "summary_large_image",
      title: name,
      description,
      images: [{ url: "/twitter-image.png", ...shareImage }],
    },
  };
}
