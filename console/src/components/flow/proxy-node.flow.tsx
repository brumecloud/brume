import { Handle, Position } from "@xyflow/react";
import type { CSSProperties } from "react";

export default function ProxyNodeFlow({
  data,
  style,
  ...props
}: {
  style: CSSProperties;
  width: number;
  height: number;
  data: {
    label: string;
  };
}) {
  return (
    <div
      style={{ ...style, width: props.width, height: props.height }}
      className="flex flex-row items-center justify-center rounded-sm border bg-white"
    >
      <p className="font-normal text-gray-800 text-xs">{data.label}</p>
      <Handle type="source" position={Position.Right} />
      <Handle type="target" style={{ opacity: 0 }} position={Position.Left} />
    </div>
  );
}
