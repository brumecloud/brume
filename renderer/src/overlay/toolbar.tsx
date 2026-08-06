import { useRef, useState } from "react";
import {
  CheckCheck,
  ChevronLeft,
  Link as LinkIcon,
  MessageCircle,
  MousePointerClick,
  Share2,
  X,
} from "lucide-react";
import type { AuthMode, ToolbarApi } from "./api";
import type { ReviewStore } from "./store";
import { ACCESS_COPY } from "./access-copy";
import { ManagementView } from "./management";
import { useStore } from "./hooks/use-store";
import { BrumeLogo } from "./brume-logo";
import { Badge } from "@/overlay/ui/badge";
import { Button } from "@/overlay/ui/button";
import { Input } from "@/overlay/ui/input";
import { Separator } from "@/overlay/ui/separator";

interface ToolbarProps {
  api: ToolbarApi;
  owner: boolean;
  authMode: AuthMode | null;
  store: ReviewStore | null;
  host: HTMLElement;
}

function AccessState({ authMode }: { authMode: AuthMode }) {
  const access = ACCESS_COPY[authMode];
  const Icon = access.icon;
  return (
    <div className="flex items-center gap-2.5 rounded-lg bg-muted p-2">
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-accent text-foreground">
        <Icon className="size-4" aria-hidden />
      </span>
      <span className="flex min-w-0 flex-col">
        <span className="text-sm leading-tight font-medium">{access.label}</span>
        <span className="truncate text-xs leading-tight text-muted-foreground">
          {access.description}
        </span>
      </span>
    </div>
  );
}

function ReviewSection({ store, onNavigate }: { store: ReviewStore; onNavigate: () => void }) {
  const round = store.round;
  const open = round.status === "open";
  const [error, setError] = useState("");
  const unplaced = store.unplacedThreads();
  const finish = () => {
    if (!confirm("Finish this review? The site owner will fetch your comments.")) return;
    store.finish().catch(() => setError("Could not finish the review"));
  };
  return (
    <div className="flex flex-col gap-2">
      <p className="m-0 flex items-center gap-1.5 px-1 text-xs" aria-live="polite">
        {open ? (
          <>
            <span className="relative flex size-1.5">
              <span className="absolute inline-flex size-full animate-ping rounded-full bg-highlight/60 motion-reduce:animate-none" />
              <span className="relative inline-flex size-1.5 rounded-full bg-highlight" />
            </span>
            <span className="text-muted-foreground">
              Review round #{round.number} ·{" "}
              {round.comment_count === 1 ? "1 comment" : `${round.comment_count} comments`}
            </span>
          </>
        ) : (
          <span className="flex items-center gap-1.5 text-success">
            <CheckCheck className="size-3.5" aria-hidden />
            Review finished
          </span>
        )}
      </p>
      {open && store.needsName() && (
        <Input
          type="text"
          placeholder="Your name"
          maxLength={64}
          value={store.name}
          className="bg-input/20"
          onChange={(event) => store.setName(event.currentTarget.value)}
        />
      )}
      {open && (
        <div className="grid grid-cols-2 gap-2">
          <Button
            type="button"
            variant="secondary"
            onClick={() => {
              onNavigate();
              store.startElementMode();
            }}
          >
            <MousePointerClick data-icon="inline-start" aria-hidden />
            Comment element
          </Button>
          <Button type="button" onClick={finish}>
            <CheckCheck data-icon="inline-start" aria-hidden />
            Finish review
          </Button>
        </div>
      )}
      {unplaced.length > 0 && (
        <div className="flex flex-col gap-1">
          {unplaced.map((orphan) => (
            <Button
              type="button"
              variant="ghost"
              key={orphan.id}
              className="w-full justify-start"
              onClick={() => {
                onNavigate();
                store.openThread(orphan.id);
              }}
            >
              <MessageCircle data-icon="inline-start" className="text-highlight" aria-hidden />
              <span className="truncate font-normal text-muted-foreground">
                {orphan.comments} · {orphan.preview}
              </span>
            </Button>
          ))}
        </div>
      )}
      <div aria-live="polite">
        {error && <p className="m-0 px-1 text-xs text-destructive">{error}</p>}
      </div>
    </div>
  );
}

export function Toolbar({ api, owner, authMode: initialAuthMode, store, host }: ToolbarProps) {
  useStore(store);
  const [open, setOpen] = useState(false);
  const [view, setView] = useState<"primary" | "management">("primary");
  const [result, setResult] = useState("");
  const [authMode, setAuthMode] = useState(initialAuthMode);
  const dragging = useRef(false);
  const round = store?.round;
  const showBadge = round?.status === "open" && round.comment_count > 0;

  const copy = () =>
    navigator.clipboard
      .writeText(location.href)
      .then(() => setResult("Copied"))
      .catch(() => setResult("Could not copy"));
  const share = () =>
    navigator.share ? navigator.share({ url: location.href }).catch(() => {}) : copy();
  const onDragDown = (event: React.PointerEvent<HTMLElement>) => {
    if ((event.target as Element | null)?.closest("button")) return;
    dragging.current = true;
    event.currentTarget.setPointerCapture(event.pointerId);
  };
  const onDragUp = (event: React.PointerEvent<HTMLElement>) => {
    if (!dragging.current) return;
    dragging.current = false;
    const side = event.clientX < innerWidth / 2 ? "left" : "right";
    host.dataset.side = side;
    try {
      localStorage.setItem("brume-toolbar-side", side);
    } catch {
      /* private mode */
    }
  };

  return (
    <div
      className="group relative"
      data-open={open ? "" : undefined}
      data-view={view === "management" ? "management" : undefined}
    >
      <button
        type="button"
        className="brume-launcher glass relative inline-flex size-13 cursor-pointer items-center justify-center rounded-full border border-border bg-background text-foreground shadow-[inset_0_1px_0_0_rgba(255,255,255,0.1),0_8px_28px_rgba(0,0,0,0.35)] outline-none select-none hover:-translate-y-0.5 hover:shadow-[inset_0_1px_0_0_rgba(255,255,255,0.1),0_12px_34px_rgba(0,0,0,0.42)] focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 active:translate-y-0 active:scale-[0.96]"
        aria-label="Open Brume toolbar"
        onClick={() => setOpen(true)}
      >
        <BrumeLogo className="size-6" />
        {showBadge && (
          <Badge className="brume-badge-pop absolute -top-1 -right-1 h-4.5 min-w-4.5 px-1 text-[10.5px] tabular-nums">
            {round?.comment_count}
          </Badge>
        )}
      </button>
      <div className="brume-panel glass-thick w-[min(390px,calc(100vw-32px))] rounded-2xl border border-border bg-card p-3 text-card-foreground shadow-[inset_0_1px_0_0_rgba(255,255,255,0.08),0_20px_70px_rgba(0,0,0,0.45)]">
        <header
          className="brume-stagger flex cursor-grab touch-none items-center justify-between px-1 pb-2.5 active:cursor-grabbing"
          onPointerDown={onDragDown}
          onPointerUp={onDragUp}
        >
          <span className="flex min-w-0 items-center gap-2">
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Back"
              className="hidden group-data-[view=management]:inline-flex"
              onClick={() => setView("primary")}
            >
              <ChevronLeft aria-hidden />
            </Button>
            <BrumeLogo className="size-4.5 shrink-0 group-data-[view=management]:hidden" />
            <strong className="truncate text-base font-semibold tracking-[-0.01em]">
              {view === "management" ? "Manage access" : "Brume"}
            </strong>
          </span>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label="Close"
            onClick={() => setOpen(false)}
          >
            <X aria-hidden />
          </Button>
        </header>
        <div className="brume-view group-data-[view=management]:hidden">
          {owner && (
            <div className="brume-stagger flex flex-col gap-2" style={{ "--stagger": "50ms" } as React.CSSProperties}>
              {authMode && <AccessState authMode={authMode} />}
              <div className="grid grid-cols-2 gap-2">
                <Button type="button" variant="secondary" onClick={share}>
                  <Share2 data-icon="inline-start" aria-hidden />
                  Share website
                </Button>
                <Button type="button" variant="secondary" onClick={copy}>
                  <LinkIcon data-icon="inline-start" aria-hidden />
                  Copy URL
                </Button>
              </div>
              <Button
                type="button"
                variant="secondary"
                className="w-full"
                onClick={() => setView("management")}
              >
                Manage access
              </Button>
              <div aria-live="polite">
                {result && <p className="m-0 px-1 text-xs text-success">{result}</p>}
              </div>
            </div>
          )}
          {owner && store && <Separator className="my-2.5" />}
          {store && (
            <div
              className="brume-stagger"
              style={{ "--stagger": owner ? "100ms" : "50ms" } as React.CSSProperties}
            >
              <ReviewSection store={store} onNavigate={() => setOpen(false)} />
            </div>
          )}
        </div>
        {owner && (
          <div className="brume-view hidden max-h-[min(520px,calc(100vh-92px))] overflow-y-auto group-data-[view=management]:block">
            <ManagementView
              api={api}
              active={view === "management"}
              onAuthModeChange={setAuthMode}
            />
          </div>
        )}
      </div>
    </div>
  );
}
