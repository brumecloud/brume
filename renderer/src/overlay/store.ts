import type { ReviewApi, ReviewRoundState, ReviewThread } from "./api";
import {
  type Anchor,
  anchorPreview,
  parseAnchor,
  resolveElementAnchor,
  resolveTextAnchor,
} from "./anchors";

const NAME_STORAGE_KEY = "brume-review-name";

export type Placement =
  | { kind: "text"; range: Range }
  | { kind: "element"; element: Element };

export type PopoverState =
  | { mode: "thread"; threadId: string; x: number; y: number }
  | { mode: "compose"; anchor: Anchor; x: number; y: number };

export interface PendingSelection {
  range: Range;
  x: number;
  y: number;
}

export function pagePath(): string {
  return location.pathname || "/";
}

export function insideOverlay(target: EventTarget | null): boolean {
  if (!(target instanceof Node)) return false;
  let node: Node | null = target;
  while (node) {
    if (
      node instanceof HTMLElement &&
      (node.dataset.brumeReviewHost !== undefined ||
        node.dataset.brumeOverlayHost !== undefined)
    ) {
      return true;
    }
    node = node.parentNode ?? (node instanceof ShadowRoot ? node.host : null);
  }
  return false;
}

function eventInsideOverlay(event: Event): boolean {
  return event
    .composedPath()
    .some(
      (node) =>
        node instanceof HTMLElement &&
        (node.dataset.brumeReviewHost !== undefined ||
          node.dataset.brumeOverlayHost !== undefined),
    );
}

export class ReviewStore {
  round: ReviewRoundState;
  threads: ReviewThread[] = [];
  placements: ReadonlyMap<string, Placement | null> = new Map();
  popover: PopoverState | null = null;
  selection: PendingSelection | null = null;
  hoveredElement: Element | null = null;
  elementMode = false;

  private listeners = new Set<() => void>();
  private version = 0;

  constructor(
    readonly api: ReviewApi,
    initialRound: ReviewRoundState,
    readonly identified: boolean,
    readonly owner: boolean,
  ) {
    this.round = initialRound;
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  getVersion = (): number => this.version;

  private notify(): void {
    this.version += 1;
    for (const listener of this.listeners) listener();
  }

  isOpen(): boolean {
    return this.round.status === "open";
  }

  needsName(): boolean {
    return !this.identified && !this.owner;
  }

  storedName(): string {
    try {
      return localStorage.getItem(NAME_STORAGE_KEY) ?? "";
    } catch {
      return "";
    }
  }

  rememberName(name: string): void {
    try {
      localStorage.setItem(NAME_STORAGE_KEY, name);
    } catch {
      /* private mode */
    }
  }

  async refresh(): Promise<void> {
    const response = await this.api.threads();
    if (response.round) this.round = response.round;
    this.threads = response.threads;
    this.relayout();
  }

  relayout(): void {
    const placements = new Map<string, Placement | null>();
    for (const thread of this.threads) {
      if (thread.page_path !== pagePath()) continue;
      placements.set(thread.id, this.computePlacement(thread));
    }
    this.placements = placements;
    this.notify();
  }

  private computePlacement(thread: ReviewThread): Placement | null {
    const anchor = parseAnchor(thread.anchor);
    if (!anchor) return null;
    if (anchor.type === "text") {
      const range = resolveTextAnchor(anchor);
      return range ? { kind: "text", range } : null;
    }
    const element = resolveElementAnchor(anchor);
    return element ? { kind: "element", element } : null;
  }

  unplacedThreads(): { id: string; preview: string; comments: number }[] {
    return this.threads
      .filter(
        (thread) =>
          thread.page_path === pagePath() && this.placements.get(thread.id) === null,
      )
      .map((thread) => {
        const anchor = parseAnchor(thread.anchor);
        return {
          id: thread.id,
          preview: anchor ? anchorPreview(anchor) : "Comment",
          comments: thread.comments.length,
        };
      });
  }

  openThread(threadId: string, x?: number, y?: number): void {
    this.popover = {
      mode: "thread",
      threadId,
      x: x ?? scrollX + 32,
      y: y ?? scrollY + 48,
    };
    this.notify();
  }

  openComposer(anchor: Anchor, x: number, y: number): void {
    this.popover = { mode: "compose", anchor, x, y };
    this.notify();
  }

  closePopover(): void {
    this.popover = null;
    this.notify();
  }

  clearSelection(): void {
    this.selection = null;
    this.notify();
  }

  startElementMode(): void {
    if (!this.isOpen()) return;
    this.popover = null;
    this.selection = null;
    this.elementMode = true;
    document.documentElement.style.cursor = "crosshair";
    this.notify();
  }

  exitElementMode(): void {
    this.elementMode = false;
    this.hoveredElement = null;
    document.documentElement.style.cursor = "";
    this.notify();
  }

  async createThread(anchor: Anchor, body: string, name?: string): Promise<void> {
    if (name) this.rememberName(name);
    await this.api.createThread(pagePath(), anchor, body, name);
    this.popover = null;
    this.notify();
    await this.refresh();
  }

  async reply(threadId: string, body: string, name?: string): Promise<void> {
    if (name) this.rememberName(name);
    await this.api.reply(threadId, body, name);
    await this.refresh();
  }

  async finish(): Promise<void> {
    await this.api.finish();
    this.round = { ...this.round, status: "finished" };
    this.popover = null;
    this.selection = null;
    this.notify();
    this.exitElementMode();
  }

  setupListeners(): void {
    document.addEventListener("pointerup", (event) => {
      if (eventInsideOverlay(event)) return;
      setTimeout(() => this.captureSelection(), 0);
    });
    document.addEventListener("selectionchange", () => {
      const selection = getSelection();
      if ((!selection || selection.isCollapsed) && this.selection !== null) {
        this.selection = null;
        this.notify();
      }
    });
    document.addEventListener("pointerdown", (event) => {
      if (eventInsideOverlay(event)) return;
      if (this.popover) this.closePopover();
    });
    document.addEventListener(
      "pointermove",
      (event) => {
        if (!this.elementMode) return;
        const target = event.target;
        const hovered =
          target instanceof Element && !insideOverlay(target) ? target : null;
        if (hovered !== this.hoveredElement) {
          this.hoveredElement = hovered;
          this.notify();
        }
      },
      true,
    );
    document.addEventListener(
      "click",
      (event) => {
        if (!this.elementMode) return;
        const target = event.target;
        if (!(target instanceof Element) || insideOverlay(target)) return;
        event.preventDefault();
        event.stopPropagation();
        this.exitElementMode();
        const rect = target.getBoundingClientRect();
        this.pickedElement?.(target, rect.left + scrollX, rect.bottom + scrollY);
      },
      true,
    );
    document.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") return;
      if (this.elementMode) this.exitElementMode();
      else if (this.popover) this.closePopover();
    });
    let resizeTimer: ReturnType<typeof setTimeout> | undefined;
    addEventListener("resize", () => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => this.relayout(), 120);
    });
    addEventListener("popstate", () => this.relayout());
  }

  /** Wired by the review layer so element picks open a composer with a serialized anchor. */
  pickedElement: ((element: Element, x: number, y: number) => void) | null = null;

  private captureSelection(): void {
    if (!this.isOpen() || this.elementMode) return;
    const selection = getSelection();
    if (!selection || selection.isCollapsed || selection.rangeCount === 0) return;
    const range = selection.getRangeAt(0);
    if (insideOverlay(range.commonAncestorContainer)) return;
    if (!range.toString().trim()) return;
    const rects = range.getClientRects();
    const last = rects[rects.length - 1];
    if (!last) return;
    this.selection = {
      range: range.cloneRange(),
      x: last.right + scrollX + 6,
      y: last.bottom + scrollY + 6,
    };
    this.notify();
  }
}
