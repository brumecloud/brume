import { Panel, useReactFlow } from "@xyflow/react";
import { LayoutGridIcon, MaximizeIcon, MinusIcon, PlusIcon } from "lucide-react";
import type { ReactNode } from "react";

import { forgetNodePositions } from "~/api/canvas";
import type { Service } from "~/api/types";
import { Button } from "~/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "~/components/ui/tooltip";
import { canvasNodes } from "~/helpers/canvas";

export function CanvasControls({ services }: { readonly services: readonly Service[] }) {
  const { fitView, setNodes, zoomIn, zoomOut } = useReactFlow();

  return (
    <Panel position="bottom-left" className="flex flex-col gap-1">
      <ControlButton label="Zoom in" onClick={() => zoomIn()}>
        <PlusIcon />
      </ControlButton>
      <ControlButton label="Zoom out" onClick={() => zoomOut()}>
        <MinusIcon />
      </ControlButton>
      <ControlButton label="Fit view" onClick={() => fitView({ maxZoom: 1 })}>
        <MaximizeIcon />
      </ControlButton>
      <ControlButton
        label="Reset layout"
        onClick={() => {
          forgetNodePositions(services.map((service) => service.id));
          setNodes(canvasNodes(services, []));
          void fitView({ maxZoom: 1 });
        }}
      >
        <LayoutGridIcon />
      </ControlButton>
    </Panel>
  );
}

interface ControlButtonProps {
  readonly label: string;
  readonly onClick: () => void;
  readonly children: ReactNode;
}

function ControlButton({ label, onClick, children }: ControlButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={<Button variant="outline" size="icon-sm" aria-label={label} onClick={onClick} />}
      >
        {children}
      </TooltipTrigger>
      <TooltipPopup side="right">{label}</TooltipPopup>
    </Tooltip>
  );
}
