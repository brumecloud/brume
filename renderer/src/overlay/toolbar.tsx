import { useEffect, useRef, useState } from "react";
import {
  CheckCheck,
  ChevronLeft,
  Cloud,
  Link as LinkIcon,
  MessageCircle,
  MousePointerClick,
  Share2,
  X,
} from "lucide-react";
import type { ReviewStore } from "./store";
import { useStore } from "./hooks/use-store";
import { Badge } from "@/overlay/ui/badge";
import { Button } from "@/overlay/ui/button";
import { Separator } from "@/overlay/ui/separator";

interface ToolbarProps {
  site: string;
  authOrigin: string;
  owner: boolean;
  store: ReviewStore | null;
  host: HTMLElement;
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
      <p className="m-0 px-1 text-sm" aria-live="polite">
        {open ? (
          <span className="text-muted-foreground">
            Review round #{round.number} -{" "}
            {round.comment_count === 1 ? "1 comment" : `${round.comment_count} comments`}
          </span>
        ) : (
          <span className="flex items-center gap-1.5 text-success">
            <CheckCheck className="size-3.5" aria-hidden />
            Review finished
          </span>
        )}
      </p>
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
              <span className="truncate text-muted-foreground">
                {orphan.comments} · {orphan.preview}
              </span>
            </Button>
          ))}
        </div>
      )}
      <p className="m-0 min-h-4 px-1 text-xs text-destructive" aria-live="polite">
        {error}
      </p>
    </div>
  );
}

export function Toolbar({ site, authOrigin, owner, store, host }: ToolbarProps) {
  useStore(store);
  const [open, setOpen] = useState(false);
  const [view, setView] = useState<"primary" | "management">("primary");
  const [result, setResult] = useState("");
  const [managementUrl, setManagementUrl] = useState<string | null>(null);
  const dragging = useRef(false);
  const baseManagementUrl =
    authOrigin +
    "/toolbar/" +
    encodeURIComponent(site) +
    "?return_to=" +
    encodeURIComponent(location.href);
  const round = store?.round;
  const showBadge = round?.status === "open" && round.comment_count > 0;

  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (
        event.origin !== location.origin ||
        (event.data as { type?: string } | null)?.type !== "brume-owner-authenticated"
      ) {
        return;
      }
      setManagementUrl(baseManagementUrl + "&refresh=" + Date.now());
    };
    addEventListener("message", onMessage);
    return () => removeEventListener("message", onMessage);
  }, [baseManagementUrl]);

  const copy = () =>
    navigator.clipboard
      .writeText(location.href)
      .then(() => setResult("Copied"))
      .catch(() => setResult("Could not copy"));
  const share = () =>
    navigator.share ? navigator.share({ url: location.href }).catch(() => {}) : copy();
  const openManagement = () => {
    if (!managementUrl) setManagementUrl(baseManagementUrl);
    setView("management");
  };
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
        <Cloud className="mx-auto size-5.5" aria-hidden />
        {showBadge && (
          <Badge className="absolute -top-1 -right-1 h-4.5 min-w-4.5 px-1 text-[10.5px] tabular-nums">
            {round?.comment_count}
          </Badge>
        )}
      </button>
      <div className="brume-panel glass-thick w-[min(390px,calc(100vw-32px))] rounded-2xl border border-border bg-card p-2.5 text-card-foreground shadow-[inset_0_1px_0_0_rgba(255,255,255,0.08),0_20px_70px_rgba(0,0,0,0.45)]">
        <header
          className="flex cursor-grab touch-none items-center justify-between px-1 pt-0.5 pb-2.5 active:cursor-grabbing"
          onPointerDown={onDragDown}
          onPointerUp={onDragUp}
        >
          <span className="flex items-center gap-1">
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
            <strong className="text-base font-semibold tracking-[-0.01em]">
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
            <div className="flex flex-col gap-2">
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
              <Button type="button" variant="secondary" className="w-full" onClick={openManagement}>
                Manage access
              </Button>
            </div>
          )}
          {owner && store && <Separator className="my-2.5" />}
          {store && <ReviewSection store={store} onNavigate={() => setOpen(false)} />}
          <p className="m-0 mt-2 min-h-4 px-1 text-xs text-success" aria-live="polite">
            {result}
          </p>
        </div>
        {owner && (
          <div className="brume-view hidden group-data-[view=management]:block">
            {managementUrl && (
              <iframe
                title="Manage website access"
                allow="clipboard-write"
                sandbox="allow-forms allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"
                src={managementUrl}
                className="block h-[min(520px,calc(100vh-92px))] w-full rounded-lg border-0 bg-transparent"
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
