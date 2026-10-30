import { type ReactNode, createContext, use, useState } from "react";
import { createPortal } from "react-dom";

const LayerContext = createContext<HTMLElement | null>(null);

// The canvas and the panels over it. A panel stacked on another renders in the layer above
// through a portal: inside the panel below, it would be clipped by it and placed against it.
export function PanelLayer({ children }: { readonly children: ReactNode }) {
  const [layer, setLayer] = useState<HTMLElement | null>(null);

  return (
    <div className="relative h-full overflow-hidden">
      <LayerContext value={layer}>{children}</LayerContext>
      <div ref={setLayer} className="contents" />
    </div>
  );
}

export function StackedPanel({ children }: { readonly children: ReactNode }) {
  const layer = use(LayerContext);
  return layer && createPortal(children, layer);
}
