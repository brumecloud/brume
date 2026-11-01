"use client";

import { useEffect, useEffectEvent, useRef, useState, useSyncExternalStore } from "react";

import { archComponents } from "~/architecture/components";
import { architectureCopy } from "~/architecture/copy";
import { type ComponentId, type Flow, edgesOf } from "~/architecture/model";
import { type Tour, tours } from "~/architecture/tours";

import { ComponentPanel } from "./component-panel";
import { Diagram, nodeButtonId } from "./diagram";
import { TourBar, type TourState } from "./tour-bar";

// Long enough to read a one-sentence caption.
const STEP_MS = 4500;

const legend: readonly { readonly flow: Flow; readonly label: string; readonly swatch: string }[] =
  [
    { flow: "public", label: "Public traffic", swatch: "bg-orange-400" },
    { flow: "admin", label: "Administration", swatch: "bg-sky-400" },
    { flow: "socket", label: "Unix sockets", swatch: "bg-violet-400" },
    { flow: "control", label: "Control", swatch: "bg-fuchsia-400" },
    { flow: "storage", label: "State and backups", swatch: "bg-emerald-400" },
    { flow: "docker", label: "Docker", swatch: "bg-teal-400" },
  ];

function useMediaQuery(query: string): boolean {
  return useSyncExternalStore(
    (onChange) => {
      const media = window.matchMedia(query);
      media.addEventListener("change", onChange);
      return () => media.removeEventListener("change", onChange);
    },
    () => window.matchMedia(query).matches,
    () => false,
  );
}

/**
 * The interactive diagram: pick a component to read about it, or follow a tour through the
 * server. A tour moves the selection from step to step; picking a component stops the tour.
 */
export function Explorer() {
  const [picked, setPicked] = useState<ComponentId | null>(null);
  const [tour, setTour] = useState<TourState | null>(null);
  const [playing, setPlaying] = useState(false);
  const reducedMotion = useMediaQuery("(prefers-reduced-motion: reduce)");
  const wide = useMediaQuery("(min-width: 80rem)");
  const root = useRef<HTMLDivElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  // What opened the panel: a box of the diagram or a button of the phone row. Closing returns there.
  const opener = useRef<HTMLElement | null>(null);
  // Set when "Talks to" picks the next component, so the focus follows into the new panel.
  const fromPanel = useRef(false);

  const current = tour ? tours.find((candidate) => candidate.id === tour.id) : undefined;
  const step = tour && current ? current.steps[tour.step] : undefined;
  const last = current ? current.steps.length - 1 : 0;
  const autoplaying = playing && !reducedMotion && tour !== null && tour.step < last;

  const selected = step ? step.focus : picked;
  const litEdges = new Set<string>(
    step ? (step.edge ? [step.edge] : []) : picked ? edgesOf(picked).map((edge) => edge.id) : [],
  );

  function select(id: ComponentId) {
    if (!fromPanel.current && document.activeElement instanceof HTMLElement) {
      opener.current = document.activeElement;
    }
    setTour(null);
    setPlaying(false);
    setPicked(picked === id && !tour ? null : id);
  }

  function close() {
    const focusBack = selected;
    setTour(null);
    setPlaying(false);
    setPicked(null);
    const back = opener.current?.isConnected ? opener.current : null;
    opener.current = null;
    if (back) back.focus();
    else if (focusBack) document.getElementById(nodeButtonId(focusBack))?.focus();
  }

  function startTour(id: Tour["id"]) {
    setPicked(null);
    setTour({ id, step: 0 });
    setPlaying(!reducedMotion);
  }

  function goToStep(index: number) {
    if (!tour) return;
    setTour({ id: tour.id, step: index });
    setPlaying(false);
  }

  function togglePlay() {
    if (!tour) return;
    if (tour.step === last) {
      setTour({ id: tour.id, step: 0 });
      setPlaying(true);
    } else {
      setPlaying(!autoplaying);
    }
  }

  // Autoplay: one step every few seconds, until the last one.
  useEffect(() => {
    if (!autoplaying || !tour) return;
    const timer = window.setTimeout(() => setTour({ id: tour.id, step: tour.step + 1 }), STEP_MS);
    return () => window.clearTimeout(timer);
  }, [autoplaying, tour]);

  // Escape closes the panel, or stops the tour, and gives the focus back. Only while the focus is
  // in the explorer: elsewhere on the page, Escape is not ours.
  const onEscape = useEffectEvent((event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    if (root.current?.contains(document.activeElement)) close();
  });
  const open = selected !== null;
  useEffect(() => {
    if (!open) return;
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  }, [open]);

  function selectFromPanel(id: ComponentId) {
    fromPanel.current = true;
    select(id);
  }

  // A new panel starts at its title. Beside the diagram, the panel scrolls on its own; below it,
  // on narrow screens, the page scrolls to it.
  useEffect(() => {
    const element = panel.current;
    if (picked === null || !element) return;
    if (wide) element.scrollTop = 0;
    else element.scrollIntoView({ block: "start", behavior: reducedMotion ? "auto" : "smooth" });
    if (fromPanel.current) {
      fromPanel.current = false;
      element.querySelector<HTMLElement>("h3")?.focus({ preventScroll: true });
    }
  }, [picked, wide, reducedMotion]);

  return (
    <div ref={root} className="mx-auto w-full max-w-[90rem] px-6">
      <TourBar
        tour={tour}
        playing={autoplaying}
        canPlay={!reducedMotion}
        onStart={startTour}
        onStep={goToStep}
        onTogglePlay={togglePlay}
        onStop={close}
      />
      <div className="mt-6 grid gap-6 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <div className="min-w-0">
          <figure data-light className="group pixel-border bg-terminal">
            <div className="overflow-x-auto px-4 py-6 sm:px-6">
              <Diagram selected={selected} litEdges={litEdges} onSelect={select} />
            </div>
            <figcaption className="flex flex-wrap items-center gap-x-5 gap-y-2 border-t-2 border-edge px-6 py-3.5 text-[13px] text-ink-muted">
              <span className="basis-full text-ink-dim">{architectureCopy.modes}</span>
              {legend.map((item) => (
                <span key={item.flow} className="flex items-center gap-2">
                  <span aria-hidden="true" className={`size-2 ${item.swatch}`} />
                  {item.label}
                </span>
              ))}
            </figcaption>
          </figure>
          {/* On a phone the boxes are small: the same components, as plain buttons. */}
          <ul className="mt-4 flex flex-wrap gap-2 sm:hidden">
            {archComponents.map((component) => (
              <li key={component.id}>
                <button
                  type="button"
                  aria-pressed={selected === component.id}
                  onClick={() => select(component.id)}
                  className="pixel-border min-h-11 cursor-pointer bg-surface px-3 text-sm aria-pressed:bg-highlight aria-pressed:text-brand"
                >
                  {component.label}
                </button>
              </li>
            ))}
          </ul>
        </div>
        {/* Beside the diagram, the panel takes its height and scrolls inside, never taller. */}
        <div className="relative">
          <div
            ref={panel}
            className="scroll-mt-6 [scrollbar-width:thin] xl:absolute xl:inset-0 xl:overflow-y-auto"
          >
            {selected ? (
              <ComponentPanel id={selected} onSelect={selectFromPanel} onClose={close} />
            ) : (
              <div className="pixel-border hidden h-full bg-surface p-6 text-ink-muted xl:block">
                <p className="text-label text-ink-dim">{architectureCopy.panelHintTitle}</p>
                <p className="mt-3 leading-relaxed text-pretty">{architectureCopy.panelHint}</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
