import type { ReviewApi, ReviewRoundState, ReviewThread } from "./api";
import {
  type Anchor,
  anchorFromElement,
  anchorFromSelection,
  anchorPreview,
  parseAnchor,
  resolveElementAnchor,
  resolveTextAnchor,
} from "./anchors";

const NAME_STORAGE_KEY = "brume-review-name";

type Placement =
  | { kind: "text"; range: Range }
  | { kind: "element"; element: Element };

type PopoverMode =
  | { mode: "thread"; threadId: string }
  | { mode: "compose"; anchor: Anchor; x: number; y: number };

export interface ReviewController {
  round(): ReviewRoundState;
  unplacedThreads(): { id: string; preview: string; comments: number }[];
  openThread(threadId: string): void;
  startElementMode(): void;
  finish(): Promise<void>;
  onChange(listener: () => void): void;
}

interface ReviewOptions {
  identified: boolean;
  owner: boolean;
}

const LAYER_STYLES = `
  *{box-sizing:border-box}
  :host{font:13px/1.4 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;color:#f5f5f5}
  .highlight{position:absolute;background:rgba(255,203,64,.32);border-bottom:2px solid #f5b700;border-radius:2px;pointer-events:none}
  .hoverbox{position:absolute;border:2px solid #8ab4f8;border-radius:6px;background:rgba(138,180,248,.12);pointer-events:none;display:none}
  .marker{position:absolute;display:flex;align-items:center;gap:4px;padding:3px 8px;border:1px solid rgba(255,255,255,.16);border-radius:999px;background:rgba(20,20,20,.95);color:#f5f5f5;font-size:12px;cursor:pointer;box-shadow:0 4px 16px rgba(0,0,0,.35);transition:transform .16s ease}
  .marker:hover{transform:scale(1.06)}
  .float{position:absolute;padding:6px 11px;border:1px solid rgba(255,255,255,.16);border-radius:999px;background:rgba(20,20,20,.95);color:#f5f5f5;font-size:12px;cursor:pointer;box-shadow:0 6px 22px rgba(0,0,0,.4);display:none}
  .float:hover{background:#2c2c2c}
  .popover{position:absolute;width:min(380px,calc(100vw - 32px));padding:12px;border:1px solid rgba(255,255,255,.13);border-radius:14px;background:rgba(18,18,18,.97);box-shadow:0 20px 70px rgba(0,0,0,.5);backdrop-filter:blur(20px);display:none;z-index:2}
  .quote{margin:0 0 10px;padding:2px 0 2px 10px;border-left:3px solid #f5b700;color:#d8d8d8;white-space:pre-wrap;word-break:break-word;max-height:76px;overflow:hidden}
  .comments{display:flex;flex-direction:column;gap:10px;max-height:280px;overflow-y:auto;margin-bottom:10px}
  .comment header{display:flex;align-items:baseline;gap:8px;margin-bottom:2px}
  .comment header strong{font-size:12.5px;font-weight:620}
  .comment header time{color:#8f8f8f;font-size:11.5px}
  .comment p{margin:0;white-space:pre-wrap;word-break:break-word}
  textarea,input{width:100%;padding:8px 10px;border:1px solid #3a3a3a;border-radius:9px;background:#232323;color:#f5f5f5;font:inherit}
  textarea{resize:vertical;min-height:56px}
  textarea:focus,input:focus{outline:2px solid #8ab4f8;outline-offset:1px}
  form{display:flex;flex-direction:column;gap:8px}
  .row{display:flex;gap:8px;justify-content:flex-end}
  button{font:inherit;border:1px solid #3a3a3a;border-radius:9px;padding:7px 12px;background:#272727;color:#f5f5f5;cursor:pointer}
  button:hover{background:#333}
  button.primary{background:#e8e8e8;border-color:#e8e8e8;color:#151515;font-weight:600}
  button.primary:hover{background:#fff}
  .error{margin:6px 0 0;color:#ff9d9d;min-height:16px}
  .finished-note{color:#9ee6bc;margin:0 0 8px}
  @media(prefers-reduced-motion:reduce){*{transition-duration:.01ms!important}}
`;

function relativeTime(iso: string): string {
  const timestamp = Date.parse(iso);
  if (Number.isNaN(timestamp)) return "";
  const seconds = Math.max(0, (Date.now() - timestamp) / 1000);
  if (seconds < 60) return "Just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function authorLabel(comment: {
  author_public_id: string | null;
  author_display_name: string | null;
  author_is_owner: boolean;
}): string {
  if (comment.author_is_owner) return "Owner";
  return comment.author_display_name || comment.author_public_id || "Anonymous";
}

export function mountReview(
  api: ReviewApi,
  initialRound: ReviewRoundState,
  options: ReviewOptions,
): ReviewController {
  const host = document.createElement("div");
  host.dataset.brumeReviewHost = "";
  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `<style>${LAYER_STYLES}</style>
    <div class="highlights"></div>
    <div class="markers"></div>
    <div class="hoverbox"></div>
    <button class="float" type="button">💬 Comment</button>
    <div class="popover"></div>`;
  document.documentElement.append(host);

  const highlights = root.querySelector(".highlights") as HTMLElement;
  const markers = root.querySelector(".markers") as HTMLElement;
  const hoverbox = root.querySelector(".hoverbox") as HTMLElement;
  const float = root.querySelector(".float") as HTMLButtonElement;
  const popover = root.querySelector(".popover") as HTMLElement;

  let round = initialRound;
  let threads: ReviewThread[] = [];
  const placements = new Map<string, Placement | null>();
  let popoverMode: PopoverMode | null = null;
  let pendingSelection: Range | null = null;
  let elementMode = false;
  const listeners: (() => void)[] = [];

  const notify = () => listeners.forEach((listener) => listener());
  const isOpen = () => round.status === "open";
  const needsName = () => !options.identified && !options.owner;

  const storedName = (): string => {
    try {
      return localStorage.getItem(NAME_STORAGE_KEY) ?? "";
    } catch {
      return "";
    }
  };

  const rememberName = (name: string) => {
    try {
      localStorage.setItem(NAME_STORAGE_KEY, name);
    } catch {
      /* private mode */
    }
  };

  const pagePath = () => location.pathname || "/";

  const insideOverlay = (target: EventTarget | null): boolean => {
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
  };

  function pageRect(rect: DOMRect): { left: number; top: number; width: number; height: number } {
    return {
      left: rect.left + scrollX,
      top: rect.top + scrollY,
      width: rect.width,
      height: rect.height,
    };
  }

  function computePlacement(thread: ReviewThread): Placement | null {
    const anchor = parseAnchor(thread.anchor);
    if (!anchor) return null;
    if (anchor.type === "text") {
      const range = resolveTextAnchor(anchor);
      return range ? { kind: "text", range } : null;
    }
    const element = resolveElementAnchor(anchor);
    return element ? { kind: "element", element } : null;
  }

  function render() {
    highlights.textContent = "";
    markers.textContent = "";
    placements.clear();
    for (const thread of threads) {
      if (thread.page_path !== pagePath()) continue;
      const placement = computePlacement(thread);
      placements.set(thread.id, placement);
      if (!placement) continue;
      let markerPoint: { x: number; y: number } | null = null;
      if (placement.kind === "text") {
        const rects = Array.from(placement.range.getClientRects());
        for (const rect of rects) {
          if (rect.width === 0 || rect.height === 0) continue;
          const box = pageRect(rect);
          const div = document.createElement("div");
          div.className = "highlight";
          div.style.left = `${box.left}px`;
          div.style.top = `${box.top}px`;
          div.style.width = `${box.width}px`;
          div.style.height = `${box.height}px`;
          highlights.append(div);
        }
        const last = rects[rects.length - 1];
        if (last) markerPoint = { x: last.right + scrollX + 6, y: last.top + scrollY - 4 };
      } else {
        const rect = placement.element.getBoundingClientRect();
        markerPoint = { x: rect.right + scrollX - 10, y: rect.top + scrollY - 10 };
      }
      if (!markerPoint) continue;
      const marker = document.createElement("button");
      marker.className = "marker";
      marker.type = "button";
      marker.textContent = `💬 ${thread.comments.length}`;
      marker.style.left = `${Math.max(8, markerPoint.x)}px`;
      marker.style.top = `${Math.max(8, markerPoint.y)}px`;
      marker.onclick = (event) => {
        event.stopPropagation();
        openThreadPopover(thread.id, markerPoint.x, markerPoint.y + 24);
      };
      markers.append(marker);
    }
    if (popoverMode?.mode === "thread") {
      const openThreadId = popoverMode.threadId;
      const thread = threads.find((thread) => thread.id === openThreadId);
      if (thread) renderThreadPopover(thread);
      else closePopover();
    }
  }

  function positionPopover(x: number, y: number) {
    const width = Math.min(380, innerWidth - 32);
    const left = Math.min(Math.max(x, scrollX + 16), scrollX + innerWidth - width - 16);
    popover.style.left = `${left}px`;
    popover.style.top = `${y + 8}px`;
    popover.style.display = "block";
  }

  function closePopover() {
    popover.style.display = "none";
    popover.textContent = "";
    popoverMode = null;
  }

  function nameField(form: HTMLFormElement): HTMLInputElement | null {
    if (!needsName()) return null;
    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "Your name (optional)";
    input.maxLength = 64;
    input.value = storedName();
    form.append(input);
    return input;
  }

  function composerForm(
    placeholder: string,
    onSubmit: (body: string, name?: string) => Promise<void>,
    onCancel?: () => void,
  ): HTMLFormElement {
    const form = document.createElement("form");
    const textarea = document.createElement("textarea");
    textarea.placeholder = placeholder;
    textarea.maxLength = 4096;
    form.append(textarea);
    const name = nameField(form);
    const row = document.createElement("div");
    row.className = "row";
    if (onCancel) {
      const cancel = document.createElement("button");
      cancel.type = "button";
      cancel.textContent = "Cancel";
      cancel.onclick = onCancel;
      row.append(cancel);
    }
    const submit = document.createElement("button");
    submit.type = "submit";
    submit.className = "primary";
    submit.textContent = "Comment";
    row.append(submit);
    form.append(row);
    const error = document.createElement("p");
    error.className = "error";
    form.append(error);
    form.onsubmit = async (event) => {
      event.preventDefault();
      const body = textarea.value.trim();
      if (!body) return;
      submit.disabled = true;
      error.textContent = "";
      try {
        const typedName = name?.value.trim();
        if (typedName) rememberName(typedName);
        await onSubmit(body, typedName || undefined);
      } catch (submitError) {
        error.textContent =
          submitError instanceof Error ? submitError.message : "Could not send the comment";
        submit.disabled = false;
        return;
      }
      submit.disabled = false;
    };
    setTimeout(() => textarea.focus(), 0);
    return form;
  }

  function renderThreadPopover(thread: ReviewThread) {
    popover.textContent = "";
    const anchor = parseAnchor(thread.anchor);
    if (anchor) {
      const quote = document.createElement("p");
      quote.className = "quote";
      quote.textContent = anchorPreview(anchor);
      popover.append(quote);
    }
    const list = document.createElement("div");
    list.className = "comments";
    for (const comment of thread.comments) {
      const item = document.createElement("div");
      item.className = "comment";
      const header = document.createElement("header");
      const author = document.createElement("strong");
      author.textContent = authorLabel(comment);
      const time = document.createElement("time");
      time.textContent = relativeTime(comment.created_at);
      header.append(author, time);
      const body = document.createElement("p");
      body.textContent = comment.body;
      item.append(header, body);
      list.append(item);
    }
    popover.append(list);
    if (isOpen()) {
      popover.append(
        composerForm("Reply…", async (body, name) => {
          await api.reply(thread.id, body, name);
          await refresh();
        }),
      );
    } else {
      const note = document.createElement("p");
      note.className = "finished-note";
      note.textContent = "This review is finished";
      popover.append(note);
    }
  }

  function openThreadPopover(threadId: string, x: number, y: number) {
    const thread = threads.find((thread) => thread.id === threadId);
    if (!thread) return;
    popoverMode = { mode: "thread", threadId };
    renderThreadPopover(thread);
    positionPopover(x, y);
  }

  function openComposer(anchor: Anchor, x: number, y: number) {
    popoverMode = { mode: "compose", anchor, x, y };
    popover.textContent = "";
    const quote = document.createElement("p");
    quote.className = "quote";
    quote.textContent = anchorPreview(anchor);
    popover.append(quote);
    popover.append(
      composerForm(
        "Add a comment…",
        async (body, name) => {
          await api.createThread(pagePath(), anchor, body, name);
          closePopover();
          await refresh();
        },
        closePopover,
      ),
    );
    positionPopover(x, y);
  }

  async function refresh() {
    const response = await api.threads();
    if (response.round) round = response.round;
    threads = response.threads;
    render();
    notify();
  }

  function hideFloat() {
    float.style.display = "none";
    pendingSelection = null;
  }

  function handleSelection() {
    if (!isOpen() || elementMode) return;
    const selection = getSelection();
    if (!selection || selection.isCollapsed || selection.rangeCount === 0) return;
    const range = selection.getRangeAt(0);
    if (insideOverlay(range.commonAncestorContainer)) return;
    if (!range.toString().trim()) return;
    const rects = range.getClientRects();
    const last = rects[rects.length - 1];
    if (!last) return;
    pendingSelection = range.cloneRange();
    float.style.left = `${last.right + scrollX + 6}px`;
    float.style.top = `${last.bottom + scrollY + 6}px`;
    float.style.display = "block";
  }

  float.onclick = (event) => {
    event.stopPropagation();
    if (!pendingSelection) return;
    const anchor = anchorFromSelection(pendingSelection);
    if (!anchor) {
      hideFloat();
      return;
    }
    const rects = pendingSelection.getClientRects();
    const last = rects[rects.length - 1];
    const x = last ? last.left + scrollX : scrollX + 32;
    const y = last ? last.bottom + scrollY : scrollY + 32;
    hideFloat();
    openComposer(anchor, x, y);
  };

  function exitElementMode() {
    elementMode = false;
    hoverbox.style.display = "none";
    document.documentElement.style.cursor = "";
  }

  function handleElementHover(event: PointerEvent) {
    if (!elementMode) return;
    const target = event.target;
    if (!(target instanceof Element) || insideOverlay(target)) {
      hoverbox.style.display = "none";
      return;
    }
    const box = pageRect(target.getBoundingClientRect());
    hoverbox.style.left = `${box.left - 2}px`;
    hoverbox.style.top = `${box.top - 2}px`;
    hoverbox.style.width = `${box.width}px`;
    hoverbox.style.height = `${box.height}px`;
    hoverbox.style.display = "block";
  }

  function handleElementPick(event: MouseEvent) {
    if (!elementMode) return;
    const target = event.target;
    if (!(target instanceof Element) || insideOverlay(target)) return;
    event.preventDefault();
    event.stopPropagation();
    exitElementMode();
    const rect = target.getBoundingClientRect();
    openComposer(anchorFromElement(target), rect.left + scrollX, rect.bottom + scrollY);
  }

  document.addEventListener("pointerup", (event) => {
    if (insideOverlay(event.target)) return;
    setTimeout(handleSelection, 0);
  });
  document.addEventListener("selectionchange", () => {
    const selection = getSelection();
    if (!selection || selection.isCollapsed) hideFloat();
  });
  document.addEventListener("pointerdown", (event) => {
    if (insideOverlay(event.target)) return;
    if (popoverMode) closePopover();
  });
  document.addEventListener("pointermove", handleElementHover, true);
  document.addEventListener("click", handleElementPick, true);
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (elementMode) exitElementMode();
    else if (popoverMode) closePopover();
  });

  let resizeTimer: ReturnType<typeof setTimeout> | undefined;
  addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(render, 120);
  });
  addEventListener("popstate", () => render());

  void refresh().catch(() => {
    /* the review layer stays empty when the fetch fails */
  });

  return {
    round: () => round,
    unplacedThreads: () =>
      threads
        .filter(
          (thread) => thread.page_path === pagePath() && placements.get(thread.id) === null,
        )
        .map((thread) => {
          const anchor = parseAnchor(thread.anchor);
          return {
            id: thread.id,
            preview: anchor ? anchorPreview(anchor) : "Comment",
            comments: thread.comments.length,
          };
        }),
    openThread: (threadId: string) => {
      openThreadPopover(threadId, scrollX + 32, scrollY + 48);
    },
    startElementMode: () => {
      if (!isOpen()) return;
      closePopover();
      hideFloat();
      elementMode = true;
      document.documentElement.style.cursor = "crosshair";
    },
    finish: async () => {
      await api.finish();
      round = { ...round, status: "finished" };
      closePopover();
      hideFloat();
      exitElementMode();
      render();
      notify();
    },
    onChange: (listener: () => void) => listeners.push(listener),
  };
}
