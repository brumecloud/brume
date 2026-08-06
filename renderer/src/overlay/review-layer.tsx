import { useEffect, useState } from "react";
import { MessageCirclePlus, CheckCheck } from "lucide-react";
import type { ReviewComment, ReviewThread } from "./api";
import {
  type Anchor,
  anchorFromElement,
  anchorFromSelection,
  anchorPreview,
  parseAnchor,
} from "./anchors";
import { type Placement, type ReviewStore, pagePath } from "./store";
import { useStore } from "./hooks/use-store";
import { Button } from "@/overlay/ui/button";
import { Field, FieldError, FieldGroup } from "@/overlay/ui/field";
import { Input } from "@/overlay/ui/input";
import { Textarea } from "@/overlay/ui/textarea";
import { Spinner } from "@/overlay/ui/spinner";

interface PageBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

function pageBox(rect: DOMRect): PageBox {
  return {
    left: rect.left + scrollX,
    top: rect.top + scrollY,
    width: rect.width,
    height: rect.height,
  };
}

function relativeTime(iso: string): string {
  const timestamp = Date.parse(iso);
  if (Number.isNaN(timestamp)) return "";
  const seconds = Math.max(0, (Date.now() - timestamp) / 1000);
  if (seconds < 60) return "Just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function authorLabel(comment: ReviewComment): string {
  if (comment.author_is_owner) return "Owner";
  return comment.author_display_name || comment.author_public_id || "Anonymous";
}

function popoverPosition(x: number, y: number): { left: string; top: string } {
  const width = Math.min(380, innerWidth - 32);
  const left = Math.min(Math.max(x, scrollX + 16), scrollX + innerWidth - width - 16);
  return { left: `${left}px`, top: `${y + 8}px` };
}

function markerPoint(placement: Placement): { x: number; y: number } | null {
  if (placement.kind === "text") {
    const rects = placement.range.getClientRects();
    const last = rects[rects.length - 1];
    if (!last) return null;
    return { x: last.right + scrollX + 6, y: last.top + scrollY - 4 };
  }
  const rect = placement.element.getBoundingClientRect();
  return { x: rect.right + scrollX - 10, y: rect.top + scrollY - 10 };
}

function Composer({
  store,
  placeholder,
  onSubmit,
  onCancel,
}: {
  store: ReviewStore;
  placeholder: string;
  onSubmit: (body: string, name?: string) => Promise<void>;
  onCancel?: () => void;
}) {
  const [body, setBody] = useState("");
  const [name, setName] = useState(store.storedName());
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    const trimmed = body.trim();
    if (!trimmed) return;
    setBusy(true);
    setError("");
    try {
      await onSubmit(trimmed, name.trim() || undefined);
      setBody("");
    } catch (submitError) {
      setError(
        submitError instanceof Error ? submitError.message : "Could not send the comment",
      );
    } finally {
      setBusy(false);
    }
  };
  return (
    <form onSubmit={submit}>
      <FieldGroup className="gap-2">
        <Field data-invalid={error ? true : undefined}>
          <Textarea
            placeholder={placeholder}
            maxLength={4096}
            value={body}
            aria-invalid={error ? true : undefined}
            className="min-h-14 resize-y bg-input/20"
            ref={(element: HTMLTextAreaElement | null) => element?.focus()}
            onChange={(event) => setBody(event.currentTarget.value)}
          />
          {store.needsName() && (
            <Input
              type="text"
              placeholder="Your name (optional)"
              maxLength={64}
              value={name}
              className="bg-input/20"
              onChange={(event) => setName(event.currentTarget.value)}
            />
          )}
          <FieldError>{error || null}</FieldError>
        </Field>
        <div className="flex justify-end gap-2">
          {onCancel && (
            <Button type="button" variant="ghost" size="sm" onClick={onCancel}>
              Cancel
            </Button>
          )}
          <Button type="submit" size="sm" disabled={busy}>
            {busy && <Spinner data-icon="inline-start" />}
            Comment
          </Button>
        </div>
      </FieldGroup>
    </form>
  );
}

function Quote({ anchor }: { anchor: Anchor }) {
  return (
    <blockquote className="mb-2.5 line-clamp-3 border-l-2 border-highlight/80 pl-2.5 text-xs leading-relaxed whitespace-pre-wrap break-words text-muted-foreground">
      {anchorPreview(anchor)}
    </blockquote>
  );
}

function ThreadPopover({ store, thread }: { store: ReviewStore; thread: ReviewThread }) {
  const anchor = parseAnchor(thread.anchor);
  return (
    <>
      {anchor && <Quote anchor={anchor} />}
      <div className="scroll-fade-y mb-2.5 flex max-h-70 flex-col gap-3 overflow-y-auto">
        {thread.comments.map((comment) => (
          <div key={comment.id} className="flex flex-col gap-0.5">
            <header className="flex items-baseline gap-2">
              <span className="text-sm font-medium">{authorLabel(comment)}</span>
              <time className="text-xs text-muted-foreground">
                {relativeTime(comment.created_at)}
              </time>
            </header>
            <p className="m-0 whitespace-pre-wrap break-words text-sm leading-relaxed text-foreground/90">
              {comment.body}
            </p>
          </div>
        ))}
      </div>
      {store.isOpen() ? (
        <Composer
          store={store}
          placeholder="Reply…"
          onSubmit={(body, name) => store.reply(thread.id, body, name)}
        />
      ) : (
        <p className="m-0 flex items-center gap-1.5 text-sm text-success">
          <CheckCheck className="size-3.5" aria-hidden />
          This review is finished
        </p>
      )}
    </>
  );
}

function ComposePopover({ store, anchor }: { store: ReviewStore; anchor: Anchor }) {
  return (
    <>
      <Quote anchor={anchor} />
      <Composer
        store={store}
        placeholder="Add a comment…"
        onSubmit={(body, name) => store.createThread(anchor, body, name)}
        onCancel={() => store.closePopover()}
      />
    </>
  );
}

export function ReviewLayer({ store }: { store: ReviewStore }) {
  useStore(store);
  const placements = store.placements;
  const threads = store.threads.filter((thread) => thread.page_path === pagePath());
  const popover = store.popover;
  const selection = store.selection;
  const hovered = store.elementMode ? store.hoveredElement : null;

  useEffect(() => {
    store.pickedElement = (element, x, y) => {
      store.openComposer(anchorFromElement(element), x, y);
    };
    return () => {
      store.pickedElement = null;
    };
  }, [store]);

  const onFloatClick = () => {
    if (!selection) return;
    const anchor = anchorFromSelection(selection.range);
    store.clearSelection();
    if (!anchor) return;
    const rects = selection.range.getClientRects();
    const last = rects[rects.length - 1];
    store.openComposer(
      anchor,
      last ? last.left + scrollX : scrollX + 32,
      last ? last.bottom + scrollY : scrollY + 32,
    );
  };

  const popoverThread =
    popover?.mode === "thread"
      ? threads.find((thread) => thread.id === popover.threadId)
      : undefined;

  return (
    <>
      {threads.map((thread) => {
        const placement = placements.get(thread.id);
        if (!placement) return null;
        const point = markerPoint(placement);
        const rects =
          placement.kind === "text"
            ? Array.from(placement.range.getClientRects()).filter(
                (rect) => rect.width > 0 && rect.height > 0,
              )
            : [];
        return (
          <div key={thread.id} className="contents">
            {rects.map((rect, index) => {
              const box = pageBox(rect);
              return (
                <div
                  key={index}
                  className="pointer-events-none absolute rounded-[2px] border-b-2 border-highlight bg-highlight/25"
                  style={{
                    left: `${box.left}px`,
                    top: `${box.top}px`,
                    width: `${box.width}px`,
                    height: `${box.height}px`,
                  }}
                />
              );
            })}
            {point && (
              <button
                type="button"
                className="brume-chip-in glass absolute flex h-6 cursor-pointer items-center gap-1 rounded-full border border-border bg-popover px-2 text-xs font-medium text-foreground shadow-lg outline-none select-none hover:scale-105 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:scale-[0.97]"
                style={{
                  left: `${Math.max(8, point.x)}px`,
                  top: `${Math.max(8, point.y)}px`,
                }}
                onClick={(event) => {
                  event.stopPropagation();
                  store.openThread(thread.id, point.x, point.y + 24);
                }}
              >
                <MessageCirclePlus className="size-3 text-highlight" aria-hidden />
                {thread.comments.length}
              </button>
            )}
          </div>
        );
      })}
      {hovered &&
        (() => {
          const box = pageBox(hovered.getBoundingClientRect());
          return (
            <div
              className="pointer-events-none absolute rounded-md border-2 border-primary/80 bg-primary/10 shadow-[0_0_0_4px_rgba(255,255,255,0.06)]"
              style={{
                left: `${box.left - 2}px`,
                top: `${box.top - 2}px`,
                width: `${box.width}px`,
                height: `${box.height}px`,
              }}
            />
          );
        })()}
      {selection && !popover && (
        <button
          type="button"
          className="brume-chip-in glass absolute flex h-7 cursor-pointer items-center gap-1.5 rounded-full border border-border bg-popover px-3 text-xs font-medium text-foreground shadow-lg outline-none select-none hover:scale-[1.03] focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:scale-[0.97]"
          style={{ left: `${selection.x}px`, top: `${selection.y}px` }}
          onClick={onFloatClick}
        >
          <MessageCirclePlus className="size-3.5 text-highlight" aria-hidden />
          Comment
        </button>
      )}
      {popover && (
        <div
          className="brume-pop-in glass-thick absolute z-10 w-[min(380px,calc(100vw-32px))] rounded-xl border border-border bg-popover p-3 text-sm text-popover-foreground shadow-[inset_0_1px_0_0_rgba(255,255,255,0.08),0_20px_70px_rgba(0,0,0,0.5)]"
          style={popoverPosition(popover.x, popover.y)}
        >
          {popover.mode === "compose" ? (
            <ComposePopover store={store} anchor={popover.anchor} />
          ) : popoverThread ? (
            <ThreadPopover store={store} thread={popoverThread} />
          ) : null}
        </div>
      )}
    </>
  );
}
