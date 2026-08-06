import { useEffect, useRef, useState } from "react";
import { MessageCirclePlus, CheckCheck, Pencil, Trash2 } from "lucide-react";
import type { ReviewComment, ReviewThread } from "./api";
import {
  type Anchor,
  anchorFromElement,
  anchorFromSelection,
  anchorPreview,
  parseAnchor,
} from "./anchors";
import { type ReviewStore, pagePath } from "./store";
import { useStore } from "./hooks/use-store";
import { Button } from "@/overlay/ui/button";
import { Field, FieldError, FieldGroup } from "@/overlay/ui/field";
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

function Composer({
  store,
  placeholder,
  onSubmit,
  onCancel,
}: {
  store: ReviewStore;
  placeholder: string;
  onSubmit: (body: string) => Promise<void>;
  onCancel?: () => void;
}) {
  const [body, setBody] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const submit = async () => {
    const trimmed = body.trim();
    if (!trimmed) return;
    setBusy(true);
    setError("");
    try {
      await onSubmit(trimmed);
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
    <form
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
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
            onKeyDown={(event) => {
              if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
                event.preventDefault();
                void submit();
              }
            }}
          />
          {store.needsName() && !store.name.trim() && (
            <p className="m-0 px-1 text-xs text-muted-foreground">
              Commenting as Anonymous - set your name in the toolbar
            </p>
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

function Quote({ anchor, className }: { anchor: Anchor; className?: string }) {
  return (
    <blockquote
      className={`mb-2.5 line-clamp-3 border-l-2 border-highlight/80 pl-2.5 text-xs leading-relaxed whitespace-pre-wrap break-words text-muted-foreground ${className ?? ""}`}
    >
      {anchorPreview(anchor)}
    </blockquote>
  );
}

const HOLD_TO_DELETE_MS = 600;

/** Trash button armed by holding it down: the fill sweeps across while the
 *  pointer stays pressed and the delete fires when the sweep completes. */
function HoldToDelete({
  label,
  className,
  iconClassName,
  onDelete,
}: {
  label: string;
  className?: string;
  iconClassName?: string;
  onDelete: () => Promise<void>;
}) {
  const [holding, setHolding] = useState(false);
  const [busy, setBusy] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  const start = () => {
    if (busy) return;
    setHolding(true);
    timer.current = window.setTimeout(async () => {
      setHolding(false);
      setBusy(true);
      try {
        await onDelete();
      } finally {
        setBusy(false);
      }
    }, HOLD_TO_DELETE_MS);
  };
  const cancel = () => {
    window.clearTimeout(timer.current);
    setHolding(false);
  };
  useEffect(() => () => window.clearTimeout(timer.current), []);
  return (
    <button
      type="button"
      aria-label={label}
      title="Hold to delete"
      className={`relative isolate flex size-5 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded outline-none transition-colors select-none focus-visible:ring-2 focus-visible:ring-ring/50 ${holding ? "text-destructive" : "text-muted-foreground hover:text-destructive"} ${className ?? ""}`}
      onPointerDown={start}
      onPointerUp={cancel}
      onPointerLeave={cancel}
      onPointerCancel={cancel}
      onKeyDown={(event) => {
        if (event.repeat) return;
        if (event.key === "Enter" || event.key === " ") start();
      }}
      onKeyUp={cancel}
    >
      <span
        aria-hidden
        className="absolute inset-0 -z-10 origin-left bg-destructive/25"
        style={{
          transform: holding ? "scaleX(1)" : "scaleX(0)",
          transition: holding
            ? `transform ${HOLD_TO_DELETE_MS}ms linear`
            : "transform 120ms ease-out",
        }}
      />
      {busy ? (
        <Spinner className="size-3" />
      ) : (
        <Trash2 className={iconClassName ?? "size-3"} aria-hidden />
      )}
    </button>
  );
}

function CommentActions({
  canEdit,
  onEdit,
  onDelete,
}: {
  canEdit: boolean;
  onEdit: () => void;
  onDelete: () => Promise<void>;
}) {
  return (
    <span className="ml-auto flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
      {canEdit && (
        <button
          type="button"
          aria-label="Edit the comment"
          className="flex size-5 shrink-0 cursor-pointer items-center justify-center rounded text-muted-foreground transition-colors hover:text-foreground"
          onClick={onEdit}
        >
          <Pencil className="size-3" aria-hidden />
        </button>
      )}
      <HoldToDelete label="Delete the comment" onDelete={onDelete} />
    </span>
  );
}

function CommentEditor({
  store,
  comment,
  onClose,
}: {
  store: ReviewStore;
  comment: ReviewComment;
  onClose: () => void;
}) {
  const [body, setBody] = useState(comment.body);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const save = async () => {
    const trimmed = body.trim();
    if (!trimmed) return;
    setBusy(true);
    setError("");
    try {
      await store.editComment(comment.id, trimmed);
      onClose();
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the comment");
    } finally {
      setBusy(false);
    }
  };
  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <FieldGroup className="gap-2">
        <Field data-invalid={error ? true : undefined}>
          <Textarea
            maxLength={4096}
            value={body}
            aria-invalid={error ? true : undefined}
            className="min-h-14 resize-y bg-input/20"
            ref={(element: HTMLTextAreaElement | null) => element?.focus()}
            onChange={(event) => setBody(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
                event.preventDefault();
                void save();
              }
            }}
          />
          <FieldError>{error || null}</FieldError>
        </Field>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" size="sm" disabled={busy}>
            {busy && <Spinner data-icon="inline-start" />}
            Save
          </Button>
        </div>
      </FieldGroup>
    </form>
  );
}

function CommentItem({ store, comment }: { store: ReviewStore; comment: ReviewComment }) {
  const [editing, setEditing] = useState(false);
  const [deleteError, setDeleteError] = useState("");
  const canEdit = store.isOpen() && store.ownComment(comment);
  const canDelete = store.canDeleteComment(comment);
  const remove = async () => {
    setDeleteError("");
    try {
      await store.deleteComment(comment.id);
    } catch (removeError) {
      setDeleteError(
        removeError instanceof Error ? removeError.message : "Could not delete the comment",
      );
    }
  };
  return (
    <div className="group flex flex-col gap-0.5">
      <header className="flex items-baseline gap-2">
        <span className="text-sm font-medium">{authorLabel(comment)}</span>
        <time className="text-xs text-muted-foreground">
          {relativeTime(comment.created_at)}
        </time>
        {comment.edited_at && (
          <span className="text-xs text-muted-foreground">(edited)</span>
        )}
        {canDelete && !editing && (
          <CommentActions canEdit={canEdit} onEdit={() => setEditing(true)} onDelete={remove} />
        )}
      </header>
      {editing ? (
        <CommentEditor store={store} comment={comment} onClose={() => setEditing(false)} />
      ) : (
        <p className="m-0 whitespace-pre-wrap break-words text-sm leading-relaxed text-foreground/90">
          {comment.body}
        </p>
      )}
      {deleteError && <p className="m-0 text-xs text-destructive">{deleteError}</p>}
    </div>
  );
}

function ThreadPopover({ store, thread }: { store: ReviewStore; thread: ReviewThread }) {
  const anchor = parseAnchor(thread.anchor);
  const canDeleteThread = store.canDeleteThread(thread);
  return (
    <>
      {canDeleteThread && (
        <HoldToDelete
          label="Delete the thread"
          className="absolute top-2.5 right-2.5 size-6"
          iconClassName="size-3.5"
          onDelete={() => store.deleteThread(thread.id)}
        />
      )}
      {anchor && <Quote anchor={anchor} className={canDeleteThread ? "pr-8" : undefined} />}
      <div className="scroll-fade-y mb-2.5 flex max-h-70 flex-col gap-3 overflow-y-auto">
        {thread.comments.map((comment) => (
          <CommentItem key={comment.id} store={store} comment={comment} />
        ))}
      </div>
      {store.isOpen() ? (
        <Composer
          store={store}
          placeholder="Reply…"
          onSubmit={(body) => store.reply(thread.id, body)}
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
        onSubmit={(body) => store.createThread(anchor, body)}
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
        if (placement.kind === "element") {
          const box = pageBox(placement.element.getBoundingClientRect());
          return (
            <div
              key={thread.id}
              className="absolute cursor-pointer rounded-md border-2 border-highlight/70 bg-highlight/10 transition-colors hover:bg-highlight/20"
              style={{
                left: `${box.left - 2}px`,
                top: `${box.top - 2}px`,
                width: `${box.width}px`,
                height: `${box.height}px`,
              }}
              onClick={(event) => {
                event.stopPropagation();
                store.openThread(thread.id, box.left, box.top + box.height + 6);
              }}
            />
          );
        }
        const rects = Array.from(placement.range.getClientRects()).filter(
          (rect) => rect.width > 0 && rect.height > 0,
        );
        return (
          <div key={thread.id} className="contents">
            {rects.map((rect, index) => {
              const box = pageBox(rect);
              return (
                <div
                  key={index}
                  className="absolute cursor-pointer rounded-[2px] border-b-2 border-highlight bg-highlight/25 transition-colors hover:bg-highlight/40"
                  style={{
                    left: `${box.left}px`,
                    top: `${box.top}px`,
                    width: `${box.width}px`,
                    height: `${box.height}px`,
                  }}
                  onClick={(event) => {
                    event.stopPropagation();
                    store.openThread(thread.id, box.left, box.top + box.height + 6);
                  }}
                />
              );
            })}
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
