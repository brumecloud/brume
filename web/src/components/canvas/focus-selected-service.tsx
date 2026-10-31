import { useParams } from "@tanstack/react-router";
import { useReactFlow, useStore } from "@xyflow/react";
import { useEffect } from "react";

import { panelFocusCenter } from "~/helpers/canvas";

// Rendered inside React Flow: moves the viewport so the service of the open panel stays visible.
export function FocusSelectedService() {
  const { serviceId } = useParams({ strict: false });
  const { getNode, getZoom, setCenter } = useReactFlow();
  const viewportWidth = useStore((state) => state.width);

  // oxlint-disable-next-line brume/no-unreasoned-effect -- syncs the React Flow viewport with the service selected in the URL
  useEffect(() => {
    const node = serviceId ? getNode(serviceId) : undefined;
    if (node) {
      const zoom = getZoom();
      const { x, y } = panelFocusCenter(
        { position: node.position, width: node.measured?.width, height: node.measured?.height },
        viewportWidth,
        zoom,
      );
      void setCenter(x, y, { zoom, duration: 250 });
    }
  }, [serviceId, viewportWidth, getNode, getZoom, setCenter]);

  return null;
}
