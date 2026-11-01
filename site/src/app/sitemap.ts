import type { MetadataRoute } from "next";

import { pages, site } from "~/content";
import { competitors } from "~/vs";

export const dynamic = "force-static";

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    { url: site.url, changeFrequency: "monthly", priority: 1 },
    ...pages.map((page) => ({
      url: `${site.url}${page.path}`,
      changeFrequency: "monthly" as const,
      priority: 0.8,
    })),
    ...competitors.map((competitor) => ({
      url: `${site.url}/vs/${competitor.slug}`,
      lastModified: competitor.checked,
      changeFrequency: "monthly" as const,
      priority: 0.6,
    })),
  ];
}
