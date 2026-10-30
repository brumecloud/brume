"use client";

import { useRef, useState } from "react";

type CopyStatus = "idle" | "copied" | "selected";

const buttonLabel: Record<CopyStatus, string> = {
  idle: "Copy",
  copied: "Copied",
  selected: "Selected",
};

const announcement: Record<CopyStatus, string> = {
  idle: "",
  copied: "Install command copied to the clipboard.",
  selected: "Install command selected. Copy it with your keyboard.",
};

interface InstallCommandProps {
  readonly command: string;
  readonly className?: string;
}

export function InstallCommand({ command, className = "" }: InstallCommandProps) {
  const codeRef = useRef<HTMLElement>(null);
  const resetRef = useRef<number>(undefined);
  const [status, setStatus] = useState<CopyStatus>("idle");

  async function copy() {
    try {
      await navigator.clipboard.writeText(command);
      setStatus("copied");
    } catch {
      // Clipboard access can be denied: select the command instead.
      if (codeRef.current) window.getSelection()?.selectAllChildren(codeRef.current);
      setStatus("selected");
    }
    window.clearTimeout(resetRef.current);
    resetRef.current = window.setTimeout(() => setStatus("idle"), 1600);
  }

  return (
    <div
      className={`pixel-border flex items-center gap-3.5 bg-terminal px-4 py-3.5 font-mono text-[13px] ${className}`}
    >
      <span aria-hidden="true" className="text-brand">
        $
      </span>
      <code
        ref={codeRef}
        className="min-w-0 flex-1 overflow-x-auto whitespace-nowrap text-ink [scrollbar-width:none]"
      >
        {command}
      </code>
      <button
        type="button"
        onClick={copy}
        className="text-pixel shrink-0 cursor-pointer border-l border-edge py-0.5 pl-3.5 text-[10px] text-ink-muted transition-colors hover:text-brand"
      >
        {buttonLabel[status]}
      </button>
      <output className="sr-only">{announcement[status]}</output>
    </div>
  );
}
