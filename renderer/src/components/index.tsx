import {
  Children,
  isValidElement,
  type AnchorHTMLAttributes,
  type ImgHTMLAttributes,
  type ReactElement,
  type ReactNode,
} from "react";
import type { MDXComponents } from "mdx/types";
import { Callout } from "./Callout";
import { Card, CardGrid, CodeGroup, Decision, FileTree, Mermaid, Risk } from "./Planning";
import { Step, Steps } from "./Steps";
import { Tab, Tabs } from "./Tabs";

export const componentNames = new Set([
  "Callout",
  "Card",
  "CardGrid",
  "CodeGroup",
  "Decision",
  "FileTree",
  "Mermaid",
  "Risk",
  "Step",
  "Steps",
  "Tab",
  "Tabs",
]);

const safeHtmlElements = new Set(["details", "summary", "kbd", "mark", "sub", "sup"]);

export function isAllowedMdxElement(name: string | null | undefined): boolean {
  return Boolean(name && (componentNames.has(name) || safeHtmlElements.has(name)));
}

function rewriteLocalReference(value: string, sourcePath: string, kind: "asset" | "page") {
  if (
    value.startsWith("http://") ||
    value.startsWith("https://") ||
    value.startsWith("mailto:") ||
    value.startsWith("#") ||
    value.startsWith("/")
  ) {
    return value;
  }

  const sourceDirectory = sourcePath.includes("/")
    ? sourcePath.slice(0, sourcePath.lastIndexOf("/"))
    : "";
  const segments = `${sourceDirectory}/${value}`.split("/");
  const normalized: string[] = [];
  for (const segment of segments) {
    if (!segment || segment === ".") continue;
    if (segment === "..") normalized.pop();
    else normalized.push(segment);
  }
  let path = normalized.join("/");

  if (kind === "page") {
    path = path
      .replace(/\.mdx?$/i, "")
      .replace(/(^|\/)index$/i, "")
      .replace(/\/$/, "");
    return path ? `__BRUME_BASE_PATH__/${path}` : "__BRUME_BASE_PATH__";
  }
  return `__BRUME_BASE_PATH__/_assets/${path}`;
}

type CodeElementProps = { className?: string; children?: ReactNode };

function Pre({ children }: { children?: ReactNode }) {
  if (isValidElement(children)) {
    const code = children as ReactElement<CodeElementProps>;
    if (code.props.className?.split(" ").includes("language-mermaid")) {
      return <div className="mermaid">{code.props.children}</div>;
    }
  }
  return <pre>{children}</pre>;
}

function textContent(value: ReactNode): string {
  return Children.toArray(value)
    .map((child) => (typeof child === "string" || typeof child === "number" ? String(child) : ""))
    .join("");
}

function slugify(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "") || "section";
}

export function createComponents(sourcePath: string): MDXComponents {
  const headingCounts = new Map<string, number>();

  function heading(level: 1 | 2 | 3 | 4 | 5 | 6) {
    return function Heading({ children }: { children?: ReactNode }) {
      const base = slugify(textContent(children));
      const count = headingCounts.get(base) ?? 0;
      headingCounts.set(base, count + 1);
      const id = count === 0 ? base : `${base}-${count + 1}`;
      const Tag = `h${level}` as keyof React.JSX.IntrinsicElements;
      return <Tag id={id}>{children}</Tag>;
    };
  }

  function Link({ href = "", ...props }: AnchorHTMLAttributes<HTMLAnchorElement>) {
    return <a href={rewriteLocalReference(href, sourcePath, "page")} {...props} />;
  }

  function Image({ src = "", alt = "", ...props }: ImgHTMLAttributes<HTMLImageElement>) {
    return (
      <img
        alt={alt}
        loading="lazy"
        src={rewriteLocalReference(src, sourcePath, "asset")}
        {...props}
      />
    );
  }

  return {
    a: Link,
    h1: heading(1),
    h2: heading(2),
    h3: heading(3),
    h4: heading(4),
    h5: heading(5),
    h6: heading(6),
    img: Image,
    pre: Pre,
    Callout,
    Card,
    CardGrid,
    CodeGroup,
    Decision,
    FileTree,
    Mermaid,
    Risk,
    Step,
    Steps,
    Tab,
    Tabs,
  };
}
