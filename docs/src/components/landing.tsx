import { useCopyButton } from "fumadocs-ui/utils/use-copy-button";
import { Check, Copy } from "lucide-react";
import type { ReactNode } from "react";
import { siteUrl } from "~/lib/shared";

/** Centered title block of a landing page. */
export function Hero({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="not-prose mx-auto flex max-w-2xl flex-col items-center gap-5 pt-10 pb-4 text-center">
      <h1 className="text-4xl font-semibold tracking-tight text-balance sm:text-5xl">{title}</h1>
      <div className="text-lg leading-relaxed text-pretty text-fd-muted-foreground">{children}</div>
    </div>
  );
}

/**
 * A one-line prompt the reader pastes into their coding agent, like
 * "Answer my questions about Brume: brume.dev/llms.txt".
 */
export function AgentPrompt({ prompt, path }: { prompt: string; path: string }) {
  const url = `${siteUrl.replace(/^https?:\/\//, "")}${path}`;
  const text = `${prompt} ${url}`;
  const [checked, onClick] = useCopyButton(() => navigator.clipboard.writeText(text));

  return (
    <div className="not-prose mx-auto mb-12 flex w-fit max-w-full items-center gap-3 rounded-full border bg-fd-card py-2 ps-5 pe-2 shadow-sm">
      <code className="truncate font-mono text-sm">{text}</code>
      <button
        type="button"
        onClick={onClick}
        aria-label={checked ? "Copied" : "Copy prompt"}
        className="inline-flex size-8 shrink-0 items-center justify-center rounded-full text-fd-muted-foreground transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground"
      >
        {checked ? <Check className="size-4" /> : <Copy className="size-4" />}
      </button>
    </div>
  );
}

/** Two-column list of links with an icon, title and description. */
export function Features({ children }: { children: ReactNode }) {
  return <div className="not-prose grid gap-x-10 gap-y-8 py-4 md:grid-cols-2">{children}</div>;
}

interface FeatureProps {
  icon: ReactNode;
  title: string;
  href: string;
  children: ReactNode;
}

export function Feature({ icon, title, href, children }: FeatureProps) {
  return (
    <a href={href} className="group flex gap-4">
      <span className="mt-0.5 shrink-0 text-fd-muted-foreground transition-colors group-hover:text-fd-primary [&_svg]:size-6">
        {icon}
      </span>
      <span className="flex flex-col gap-1">
        <span className="font-medium transition-colors group-hover:text-fd-primary">{title}</span>
        <span className="text-sm leading-relaxed text-fd-muted-foreground">{children}</span>
      </span>
    </a>
  );
}
