"use client";

import { useEffect, useEffectEvent, useLayoutEffect, useRef, useState } from "react";
import { flushSync } from "react-dom";

// The admin console of web/, built in demo mode with its mocked API: see `pnpm console`.
const CONSOLE_URL = "/demo/index.html";
// What the console posts once its first page is on screen.
const READY = "brume-console:ready";
// Shows the console anyway if that message never comes.
const READY_FALLBACK_MS = 3000;

const GROW = { duration: 320, easing: "cubic-bezier(0.2, 0.8, 0.2, 1)" };
const RADIUS = "12px";

/**
 * The real console, shot like a product photo in the page, opened full screen on click. The
 * iframe is never moved nor remounted, so the demo keeps its state between the two.
 */
export function ConsoleStudio() {
  const [fullscreen, setFullscreen] = useState(false);
  const [src, setSrc] = useState<string>();
  const [ready, setReady] = useState(false);
  const slot = useRef<HTMLDivElement>(null);
  const frame = useRef<HTMLDivElement>(null);
  const iframe = useRef<HTMLIFrameElement>(null);
  const open = useRef<HTMLButtonElement>(null);
  const back = useRef<HTMLButtonElement>(null);
  const closing = useRef(false);

  function openFullscreen() {
    setSrc(CONSOLE_URL);
    setFullscreen(true);
  }

  function close() {
    const element = frame.current;
    const target = slot.current;
    if (closing.current || !element || !target) return;
    const done = () => {
      closing.current = false;
      flushSync(() => setFullscreen(false));
      open.current?.focus();
    };
    if (prefersReducedMotion()) {
      done();
      return;
    }
    closing.current = true;
    // Shrinks back onto the shot, held there until React has put it back in the page.
    const animation = element.animate(
      [{ clipPath: "inset(0px round 0px)" }, { clipPath: insetOf(target) }],
      { ...GROW, fill: "forwards" },
    );
    void animation.finished.then(() => {
      done();
      animation.cancel();
    });
  }

  // Starts the console once the page has loaded and the browser is idle: it is ready by the time
  // the visitor scrolls here, without slowing down the top of the page.
  useEffect(() => {
    let idle: number | undefined;
    const start = () => {
      idle = whenIdle(() => setSrc(CONSOLE_URL));
    };
    if (document.readyState === "complete") start();
    else window.addEventListener("load", start, { once: true });
    return () => {
      window.removeEventListener("load", start);
      if (idle !== undefined) cancelIdle(idle);
    };
  }, []);

  // The console stays invisible until it has drawn its first page, then fades in.
  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (
        event.origin === window.location.origin &&
        event.source === iframe.current?.contentWindow &&
        event.data === READY
      ) {
        setReady(true);
      }
    };
    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }, []);

  // Grows from the shot to the whole screen, before the first full-screen frame is painted.
  useLayoutEffect(() => {
    const element = frame.current;
    const target = slot.current;
    if (!fullscreen || !element || !target || prefersReducedMotion()) return;
    element.animate([{ clipPath: insetOf(target) }, { clipPath: "inset(0px round 0px)" }], GROW);
  }, [fullscreen]);

  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape") close();
  });

  // While full screen, the page behind is inert and does not scroll. Escape only reaches the page
  // when the focus is outside the console, which keeps it for its own dialogs.
  useEffect(() => {
    const element = frame.current;
    if (!fullscreen || !element) return;
    const restoreInert = inertOutside(element);
    const root = document.documentElement;
    const overflow = root.style.overflow;
    root.style.overflow = "hidden";
    back.current?.focus();
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      root.style.overflow = overflow;
      restoreInert();
    };
  }, [fullscreen]);

  return (
    // Clips the shadow where it spills past the screen; the full-screen console is fixed, so it is
    // not clipped.
    <div className="relative mt-16 overflow-x-clip">
      {/* The studio: a backdrop nearly as wide as the screen, lit from the front. */}
      <div
        aria-hidden="true"
        className="studio-backdrop absolute inset-x-3 inset-y-0 overflow-hidden rounded-2xl"
      >
        <div className="studio-grain absolute inset-0 opacity-40" />
      </div>
      <div className="relative pb-12">
        <div
          ref={slot}
          className="relative mx-auto h-[clamp(24rem,52vw,45rem)] w-[min(calc(100%-3rem),82.5rem)] rounded-xl md:w-[min(calc(100%-7.5rem),82.5rem)]"
        >
          {/* The shadow the window casts on the floor: a thin lens along its bottom edge that
              reaches past its corners, darkest there, and faint in the light under its middle. */}
          <div
            aria-hidden="true"
            className="studio-shadow pointer-events-none absolute bottom-2 left-1/2 h-8 w-[calc(100%+2.5rem)] -translate-x-1/2 translate-y-1/2 blur-[5px] md:h-11 md:w-[calc(100%+12rem)]"
          />
          <div
            ref={frame}
            role={fullscreen ? "dialog" : undefined}
            aria-modal={fullscreen ? true : undefined}
            aria-label={fullscreen ? "Brume console demo" : undefined}
            className={
              fullscreen
                ? "fixed inset-0 z-50 flex flex-col bg-night"
                : "absolute inset-0 flex flex-col overflow-hidden rounded-xl border border-white/8 bg-surface"
            }
          >
            {fullscreen ? (
              <div className="flex items-center gap-1.5 border-b-2 border-edge px-3.5 py-2 font-mono text-[11.5px] text-ink-dim">
                <span aria-hidden="true" className="size-2 bg-edge-strong" />
                <span aria-hidden="true" className="size-2 bg-edge-strong" />
                <span aria-hidden="true" className="size-2 bg-edge-strong" />
                <span className="ml-2.5 truncate">brume console · demo data</span>
                <button
                  ref={back}
                  type="button"
                  onClick={close}
                  className="ml-auto flex shrink-0 items-center gap-2 px-2 py-1 text-ink-muted transition-colors hover:text-ink"
                >
                  ← Back to the page
                  <kbd className="pixel-border px-1.5 text-[10.5px] text-ink-dim">Esc</kbd>
                </button>
              </div>
            ) : null}
            <div className="relative min-h-0 flex-1 overflow-hidden">
              {ready ? null : (
                <p className="absolute inset-0 grid place-items-center font-mono text-[12px] text-ink-dim">
                  Starting the console…
                </p>
              )}
              {/* oxlint-disable-next-line react/iframe-missing-sandbox -- our own page on the same origin: the mocked API needs its origin for the service worker, and a sandbox allowing scripts and that origin would protect nothing */}
              <iframe
                ref={iframe}
                src={src}
                title="Brume console, running on demo data"
                // In the page the console is a picture: it runs, but takes no input until opened.
                inert={!fullscreen}
                onLoad={() => {
                  if (src) window.setTimeout(() => setReady(true), READY_FALLBACK_MS);
                }}
                // In the page, the console is drawn at 80%: it shows as much as a laptop screen.
                className={`absolute top-0 left-0 bg-night transition-opacity duration-500 ${
                  fullscreen ? "size-full" : "h-[125%] w-[125%] origin-top-left scale-80"
                } ${ready ? "opacity-100" : "opacity-0"}`}
              />
            </div>
            {fullscreen ? null : (
              <>
                <button
                  ref={open}
                  type="button"
                  onClick={openFullscreen}
                  className="group absolute inset-0 flex cursor-zoom-in items-end justify-center rounded-xl pb-8"
                >
                  <span className="rounded-full border border-white/15 bg-night/75 px-4 py-2 text-sm text-ink-muted shadow-lg backdrop-blur-sm transition-colors group-hover:text-ink">
                    Open the console <span aria-hidden="true">⤢</span>
                  </span>
                </button>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

// A clip-path that keeps only the box of `element` on screen, with its rounded corners.
function insetOf(element: HTMLElement): string {
  const box = element.getBoundingClientRect();
  return `inset(${box.top}px ${window.innerWidth - box.right}px ${window.innerHeight - box.bottom}px ${box.left}px round ${RADIUS})`;
}

// Makes everything but `element` and its ancestors inert, like a modal dialog. Returns the undo.
function inertOutside(element: HTMLElement): () => void {
  const changed: HTMLElement[] = [];
  for (let node = element; node.parentElement && node !== document.body;) {
    for (const sibling of node.parentElement.children) {
      if (sibling !== node && sibling instanceof HTMLElement && !sibling.inert) {
        sibling.inert = true;
        changed.push(sibling);
      }
    }
    node = node.parentElement;
  }
  return () => {
    for (const sibling of changed) sibling.inert = false;
  };
}

// `requestIdleCallback`, or a short timeout where the browser lacks it (Safari).
function whenIdle(callback: () => void): number {
  return typeof requestIdleCallback === "function"
    ? requestIdleCallback(callback, { timeout: 2000 })
    : window.setTimeout(callback, 200);
}

function cancelIdle(handle: number): void {
  if (typeof cancelIdleCallback === "function") cancelIdleCallback(handle);
  else window.clearTimeout(handle);
}
