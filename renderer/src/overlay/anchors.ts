export interface TextAnchor {
  type: "text";
  exact: string;
  prefix?: string;
  suffix?: string;
}

export interface ElementAnchor {
  type: "element";
  css: string;
  tag: string;
  src?: string;
  text_digest?: string;
}

export type Anchor = TextAnchor | ElementAnchor;

const CONTEXT_CHARS = 32;
const MAX_EXACT_CHARS = 512;
const DIGEST_CHARS = 64;

export function parseAnchor(value: unknown): Anchor | null {
  if (typeof value !== "object" || value === null) return null;
  const anchor = value as Record<string, unknown>;
  if (anchor.type === "text" && typeof anchor.exact === "string") {
    return anchor as unknown as TextAnchor;
  }
  if (
    anchor.type === "element" &&
    typeof anchor.css === "string" &&
    typeof anchor.tag === "string"
  ) {
    return anchor as unknown as ElementAnchor;
  }
  return null;
}

function bodyText(): string {
  const range = document.createRange();
  range.selectNodeContents(document.body);
  return range.toString();
}

function rangeStartOffset(range: Range): number {
  const prefix = document.createRange();
  prefix.selectNodeContents(document.body);
  prefix.setEnd(range.startContainer, range.startOffset);
  return prefix.toString().length;
}

export function anchorFromSelection(range: Range): TextAnchor | null {
  const raw = range.toString();
  if (!raw.trim()) return null;
  const full = bodyText();
  const start = rangeStartOffset(range);
  const exact = raw.slice(0, MAX_EXACT_CHARS);
  const anchor: TextAnchor = { type: "text", exact };
  const prefix = full.slice(Math.max(0, start - CONTEXT_CHARS), start);
  const suffix = full.slice(start + exact.length, start + exact.length + CONTEXT_CHARS);
  if (prefix) anchor.prefix = prefix;
  if (suffix) anchor.suffix = suffix;
  return anchor;
}

function rangeFromOffsets(start: number, end: number): Range | null {
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const range = document.createRange();
  let cursor = 0;
  let startPlaced = false;
  let node = walker.nextNode() as Text | null;
  while (node) {
    const next = cursor + node.data.length;
    if (!startPlaced && start >= cursor && start <= next) {
      range.setStart(node, start - cursor);
      startPlaced = true;
    }
    if (startPlaced && end >= cursor && end <= next) {
      range.setEnd(node, end - cursor);
      return range;
    }
    cursor = next;
    node = walker.nextNode() as Text | null;
  }
  return null;
}

export function resolveTextAnchor(anchor: TextAnchor): Range | null {
  if (!anchor.exact) return null;
  const full = bodyText();
  let best: number | null = null;
  let bestScore = -1;
  let index = full.indexOf(anchor.exact);
  while (index !== -1) {
    let score = 0;
    if (anchor.prefix) {
      const prefix = full.slice(Math.max(0, index - anchor.prefix.length), index);
      if (prefix === anchor.prefix) score += 2;
      else if (prefix.endsWith(anchor.prefix.trimStart())) score += 1;
    }
    if (anchor.suffix) {
      const end = index + anchor.exact.length;
      const suffix = full.slice(end, end + anchor.suffix.length);
      if (suffix === anchor.suffix) score += 2;
      else if (suffix.startsWith(anchor.suffix.trimEnd())) score += 1;
    }
    if (score > bestScore) {
      bestScore = score;
      best = index;
    }
    index = full.indexOf(anchor.exact, index + 1);
  }
  if (best === null) return null;
  return rangeFromOffsets(best, best + anchor.exact.length);
}

function cssIdentifier(value: string): string {
  return typeof CSS !== "undefined" && CSS.escape
    ? CSS.escape(value)
    : value.replace(/[^a-zA-Z0-9_-]/g, "\\$&");
}

export function cssPath(element: Element): string {
  const parts: string[] = [];
  let current: Element | null = element;
  while (current && current !== document.documentElement) {
    if (current.id) {
      parts.unshift(`#${cssIdentifier(current.id)}`);
      return parts.join(" > ");
    }
    const tag = current.tagName.toLowerCase();
    let index = 1;
    let sibling = current.previousElementSibling;
    while (sibling) {
      if (sibling.tagName === current.tagName) index += 1;
      sibling = sibling.previousElementSibling;
    }
    parts.unshift(`${tag}:nth-of-type(${index})`);
    current = current.parentElement;
  }
  parts.unshift("html");
  return parts.join(" > ");
}

export function anchorFromElement(element: Element): ElementAnchor {
  const anchor: ElementAnchor = {
    type: "element",
    css: cssPath(element).slice(0, 512),
    tag: element.tagName.toLowerCase(),
  };
  const src = element.getAttribute("src");
  if (src) anchor.src = src.slice(0, 2048);
  const digest = (element.textContent ?? "").trim().slice(0, DIGEST_CHARS);
  if (digest) anchor.text_digest = digest;
  return anchor;
}

export function resolveElementAnchor(anchor: ElementAnchor): Element | null {
  let element: Element | null = null;
  try {
    element = document.querySelector(anchor.css);
  } catch {
    return null;
  }
  if (!element || element.tagName.toLowerCase() !== anchor.tag) return null;
  if (anchor.src && element.getAttribute("src") !== anchor.src) return null;
  return element;
}

export function anchorPreview(anchor: Anchor): string {
  if (anchor.type === "text") return anchor.exact;
  return anchor.text_digest || `<${anchor.tag}>`;
}
