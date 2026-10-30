import { createGetUrl } from "fumadocs-core/source";

export const appName = "Brume";
export const docsRoute = "/docs";

/** Public origin of the docs, used in llms.txt and agent prompts. */
export const siteUrl = import.meta.env.VITE_SITE_URL ?? "http://localhost:3000";

export const gitConfig = {
  user: "brumecloud",
  repo: "brume",
  branch: "main",
  contentDir: "docs/content/docs",
};

export const repoUrl = `https://github.com/${gitConfig.user}/${gitConfig.repo}`;

const getDocsUrl = createGetUrl(docsRoute);

/** URL of the Markdown twin of a page: `/docs/a/b` → `/docs/a/b.md`. */
export function getPageMarkdownUrl(page: { slugs: string[]; locale?: string }) {
  const segments = [...page.slugs];
  if (segments.length === 0) {
    segments.push("index.md");
  } else {
    segments[segments.length - 1] += ".md";
  }

  return { segments, url: getDocsUrl(segments, page.locale) };
}

/** @returns page slugs */
export function decodeMarkdownUrl(segments: string[]) {
  if (segments.length === 0) return [];

  const out = [...segments];
  out[out.length - 1] = out[out.length - 1].replace(/\.md$/, "");
  if (out.length === 1 && out[0] === "index") out.pop();
  return out;
}
