import logo from "@/assets/logo.png";
import { Handle, Position } from "@xyflow/react";
import type { CSSProperties } from "react";

export default function IngestNodeFlow({
  data,
  style,
  ...props
}: {
  data: { label: string };
  style: CSSProperties;
  width: number;
  height: number;
}) {
  return (
    <div
      style={{ ...style, width: props.width, height: props.height }}
      className="flex h-full w-full items-center gap-x-2 rounded-sm border border-gray-100 bg-[#161616] py-1 pl-1">
      <img src={logo} className="h-8 w-8" />
      <Handle type="source" position={Position.Right} />
      <p className="text-sm font-normal text-white">{data.label}</p>
    </div>
  );
}
