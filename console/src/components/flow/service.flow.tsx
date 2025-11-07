import { Button } from "@/components/ui/button";
import { Position } from "@xyflow/react";
import { Handle } from "@xyflow/react";
import { Computer, Cpu, Drill, Globe, GlobeLock } from "lucide-react";
import type { CSSProperties } from "react";
import { useNavigate, useParams } from "react-router-dom";

export default function ServiceFlow({
  data,
  style,
  ...props
}: {
  data: { label: string };
  style: CSSProperties;
  width: number;
  height: number;
  id: string;
}) {
  const { projectId } = useParams();
  const navigate = useNavigate();
  console.log(props);
  return (
    <div
      style={{ ...style, width: props.width, height: props.height }}
      className="flex flex-col overflow-hidden rounded-sm border border-slate-200 bg-white shadow-md shadow-slate-200/30"
    >
      <Handle type="target" style={{ opacity: 0 }} position={Position.Left} />
      <div className="flex flex-col bg-green-50 p-2">
        <p className="font-normal text-gray-800 text-xs">{data.label}</p>
      </div>
      <div className="h-[1px] w-full bg-slate-200" />
      <div className="flex flex-col gap-3 p-2 py-3">
        <div className="flex flex-row items-center justify-between">
          <div className="flex flex-row items-center gap-2">
            <Cpu strokeWidth={1.5} className="h-4 w-4 text-gray-800" />
            <p className="font-normal text-xs">CPU</p>
          </div>
          <p className="font-mono text-gray-800 text-xs">0.5 CPU, 2GB RAM</p>
        </div>
        <div className="flex flex-row items-center justify-between">
          <div className="flex flex-row items-center gap-2">
            <Computer strokeWidth={1.5} className="h-4 w-4 text-gray-800" />
            <p className="font-normal text-xs">Runner</p>
          </div>
          <p className="text-gray-800 text-xs">Docker Runner</p>
        </div>
        <div className="flex flex-row items-center justify-between">
          <div className="flex flex-row items-center gap-2">
            <Drill strokeWidth={1.5} className="h-4 w-4 text-gray-800" />
            <p className="font-normal text-xs">Builder</p>
          </div>
          <p className="text-gray-800 text-xs">Docker Builder</p>
        </div>
        <div className="flex flex-row items-center justify-between">
          <div className="flex flex-row items-center gap-2">
            <Globe strokeWidth={1.5} className="h-4 w-4 text-gray-800" />
            <p className="font-normal text-xs">Network</p>
          </div>
          <p className="font-mono text-gray-800 text-xs">portfolio.brume.run</p>
        </div>
        <div className="flex flex-row items-center justify-between">
          <div className="flex flex-row items-center gap-2">
            <GlobeLock strokeWidth={1.5} className="h-4 w-4 text-gray-800" />
            <p className="font-normal text-xs">Private Net.</p>
          </div>
          <p className="font-mono text-gray-800 text-xs">
            portfolio.brume.internal
          </p>
        </div>
      </div>
      <div className="h-[1px] w-full bg-slate-200" />
      <div className="flex flex-row items-center justify-between gap-2 p-2">
        <Button
          variant="outline"
          className="w-full"
          onClick={() => {
            navigate(`/${projectId}/logs?service_id=${props.id}`);
          }}
        >
          View Logs
        </Button>
        <Button
          variant="outline"
          className="w-full"
          onClick={() => {
            navigate(`/${projectId}/services/${props.id}/runner`);
          }}
        >
          Runner Settings
        </Button>
        <Button
          variant="outline"
          className="w-full"
          onClick={() => {
            navigate(`/${projectId}/services/${props.id}/runner/variables`);
          }}
        >
          Variables
        </Button>
      </div>
    </div>
  );
}
