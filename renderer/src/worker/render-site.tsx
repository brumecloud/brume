import { createHash } from "node:crypto";
import {
  copyFile,
  lstat,
  mkdir,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";
import { evaluate } from "@mdx-js/mdx";
import rehypeShiki from "@shikijs/rehype";
import matter from "gray-matter";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";
import { renderToStaticMarkup } from "react-dom/server";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import remarkGfm from "remark-gfm";
import { unified } from "unified";
import { createComponents } from "../components";
import { PageLayout, type Heading, type NavigationItem } from "../components/PageLayout";
import type {
  AssetManifest,
  BundleManifest,
  FileManifest,
  PageManifest,
  RenderRequest,
  RenderSuccess,
} from "./protocol";
import { remarkSafeMdx } from "./safe-mdx";

const RENDERER_VERSION = "0.1.0";
const DOCUMENT_EXTENSIONS = new Set([".md", ".mdx"]);
const ASSET_CONTENT_TYPES = new Map([
  [".png", "image/png"],
  [".jpg", "image/jpeg"],
  [".jpeg", "image/jpeg"],
  [".gif", "image/gif"],
  [".webp", "image/webp"],
]);
const IGNORED_DIRECTORIES = new Set([".git", ".brume", "node_modules", "dist", "target"]);

type Document = {
  absolutePath: string;
  relativePath: string;
  route: string;
  title: string;
  body: string;
  headings: Heading[];
};

function portable(path: string): string {
  return path.split(sep).join("/");
}

function routeFor(relativePath: string, entry: string): string {
  if (relativePath === entry) return "/";
  const withoutExtension = portable(relativePath).replace(/\.mdx?$/i, "");
  const withoutIndex = withoutExtension.replace(/(^|\/)index$/i, "");
  return `/${withoutIndex}`.replace(/\/$/, "") || "/";
}

function objectPathFor(route: string): string {
  return route === "/"
    ? "routes/index.html"
    : `routes/${route.slice(1)}/index.html`;
}

function plainHeading(value: string): string {
  return value
    .replace(/<[^>]+>/g, "")
    .replace(/[`*_~]/g, "")
    .trim();
}

function slugify(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "") || "section";
}

function headingsFrom(body: string): Heading[] {
  const headings: Heading[] = [];
  const counts = new Map<string, number>();
  for (const match of body.matchAll(/^(#{1,3})\s+(.+)$/gm)) {
    const title = plainHeading(match[2] ?? "");
    const base = slugify(title);
    const count = counts.get(base) ?? 0;
    counts.set(base, count + 1);
    headings.push({
      depth: (match[1] ?? "#").length,
      id: count === 0 ? base : `${base}-${count + 1}`,
      title,
    });
  }
  return headings;
}

async function discover(root: string, directory = root): Promise<string[]> {
  const result: string[] = [];
  const entries = await readdir(directory, { withFileTypes: true });
  entries.sort((left, right) => left.name.localeCompare(right.name));
  for (const entry of entries) {
    if (entry.name.startsWith(".") && entry.name !== ".well-known") continue;
    const absolutePath = join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      throw new Error(`Symbolic links are not allowed: ${portable(relative(root, absolutePath))}`);
    }
    if (entry.isDirectory()) {
      if (!IGNORED_DIRECTORIES.has(entry.name)) {
        result.push(...(await discover(root, absolutePath)));
      }
    } else if (entry.isFile()) {
      result.push(absolutePath);
    }
  }
  return result;
}

function chooseEntry(files: string[], requested?: string): string {
  const documents = files.filter((file) => DOCUMENT_EXTENSIONS.has(extname(file).toLowerCase()));
  if (requested) {
    const normalized = portable(requested).replace(/^\.\//, "");
    if (normalized.includes("..") || normalized.startsWith("/")) {
      throw new Error("The configured entry must be a safe relative path");
    }
    if (!documents.includes(normalized)) {
      throw new Error(`Configured entry \`${normalized}\` does not exist`);
    }
    return normalized;
  }
  const preferred = ["index.mdx", "index.md", "plan.mdx", "plan.md", "README.mdx", "README.md"];
  for (const candidate of preferred) {
    const match = documents.find((file) => file.toLowerCase() === candidate.toLowerCase());
    if (match) return match;
  }
  if (documents.length === 1) return documents[0]!;
  throw new Error("No entry found. Add index.mdx, plan.md, README.md, or configure an entry");
}

async function sha256(path: string): Promise<string> {
  const data = await readFile(path);
  return createHash("sha256").update(data).digest("hex");
}

async function highlight(html: string): Promise<string> {
  const output = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeShiki, {
      defaultColor: false,
      themes: { dark: "github-dark", light: "github-light" },
    })
    .use(rehypeStringify)
    .process(html);
  return String(output);
}

async function renderDocument(
  document: Document,
  siteTitle: string,
  navigation: NavigationItem[],
): Promise<string> {
  const module = await evaluate(document.body, {
    Fragment,
    baseUrl: import.meta.url,
    format: extname(document.relativePath).toLowerCase() === ".mdx" ? "mdx" : "md",
    jsx,
    jsxs,
    remarkPlugins: [remarkGfm, remarkSafeMdx],
  });
  const Content = module.default;
  const html = renderToStaticMarkup(
    <PageLayout
      currentRoute={document.route}
      headings={document.headings}
      navigation={navigation}
      title={siteTitle}
    >
      <Content components={createComponents(document.relativePath)} />
    </PageLayout>,
  );
  return highlight(html);
}

async function copyWithManifest(
  source: string,
  destination: string,
  manifestPath: string,
): Promise<FileManifest> {
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(source, destination);
  const stats = await lstat(destination);
  return { path: manifestPath, sha256: await sha256(destination), size: stats.size };
}

export async function renderSite(request: RenderRequest): Promise<RenderSuccess> {
  const sourceRoot = resolve(request.source_dir);
  const outputRoot = resolve(request.output_dir);
  const outputRelativeToSource = portable(relative(sourceRoot, outputRoot));
  if (
    outputRoot === sourceRoot ||
    (outputRoot.startsWith(`${sourceRoot}${sep}`) && !outputRelativeToSource.startsWith(".brume/"))
  ) {
    throw new Error("Renderer output inside a plan must live below its .brume directory");
  }

  await rm(outputRoot, { force: true, recursive: true });
  await mkdir(outputRoot, { recursive: true });

  const absoluteFiles = await discover(sourceRoot);
  const relativeFiles = absoluteFiles.map((path) => portable(relative(sourceRoot, path)));
  const entry = chooseEntry(relativeFiles, request.entry);
  const routeSet = new Set<string>();
  const documents: Document[] = [];

  for (let index = 0; index < absoluteFiles.length; index += 1) {
    const absolutePath = absoluteFiles[index]!;
    const relativePath = relativeFiles[index]!;
    if (!DOCUMENT_EXTENSIONS.has(extname(relativePath).toLowerCase())) continue;
    const source = await readFile(absolutePath, "utf8");
    const parsed = matter(source);
    const headings = headingsFrom(parsed.content);
    const title =
      typeof parsed.data.title === "string"
        ? parsed.data.title
        : headings.find((heading) => heading.depth === 1)?.title ?? basename(relativePath, extname(relativePath));
    const route = routeFor(relativePath, entry);
    if (routeSet.has(route)) throw new Error(`Multiple documents resolve to route \`${route}\``);
    routeSet.add(route);
    documents.push({ absolutePath, body: parsed.content, headings, relativePath, route, title });
  }

  documents.sort((left, right) => {
    if (left.route === "/") return -1;
    if (right.route === "/") return 1;
    return left.route.localeCompare(right.route);
  });
  const root = documents.find((document) => document.route === "/");
  if (!root) throw new Error("The selected entry did not produce a root page");
  const siteTitle = request.title?.trim() || root.title;
  const navigation = documents.map(({ route, title }) => ({ route, title }));
  const pages: PageManifest[] = [];
  const sources: FileManifest[] = [];

  for (const document of documents) {
    const objectPath = objectPathFor(document.route);
    const html = await renderDocument(document, siteTitle, navigation);
    await mkdir(dirname(join(outputRoot, objectPath)), { recursive: true });
    await writeFile(join(outputRoot, objectPath), html, "utf8");
    pages.push({
      object_path: objectPath,
      route: document.route,
      source_path: document.relativePath,
      title: document.title,
    });
    sources.push(
      await copyWithManifest(
        document.absolutePath,
        join(outputRoot, "source", document.relativePath),
        `source/${document.relativePath}`,
      ),
    );
  }

  const assets: AssetManifest[] = [];
  for (let index = 0; index < absoluteFiles.length; index += 1) {
    const source = absoluteFiles[index]!;
    const relativePath = relativeFiles[index]!;
    const extension = extname(relativePath).toLowerCase();
    const contentType = ASSET_CONTENT_TYPES.get(extension);
    if (!contentType) continue;
    const file = await copyWithManifest(
      source,
      join(outputRoot, "assets", relativePath),
      `assets/${relativePath}`,
    );
    assets.push({ ...file, content_type: contentType });
  }

  const manifest: BundleManifest = {
    assets,
    entry,
    format_version: 1,
    html_contract_version: 1,
    pages,
    renderer_version: RENDERER_VERSION,
    sources,
    title: siteTitle,
  };
  const manifestPath = join(outputRoot, "brume-manifest.json");
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  return { ok: true, asset_count: assets.length, manifest_path: manifestPath, page_count: pages.length };
}
