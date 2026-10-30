import type { ReactNode } from "react";

function Public({ children }: { readonly children: ReactNode }) {
  return <span className="text-ink">{children}</span>;
}

function Admin({ children }: { readonly children: ReactNode }) {
  return <span className="text-brand-soft">{children}</span>;
}

function Core({ children }: { readonly children: ReactNode }) {
  return <span className="text-brand">{children}</span>;
}

const pad = (columns: number) => " ".repeat(columns);

// Box-drawing art: the connectors line up at columns 46, 63, 65 and 67.
const lines: readonly ReactNode[] = [
  <>
    {"  "}
    <Public>internet</Public>
    {" ──► cloudflare tunnel ──► traefik ─┬─► "}
    <Public>your apps, static sites</Public>
  </>,
  <>
    {pad(46)}
    {"└─► "}
    <Public>brume webhook</Public>
    {" ─┐"}
  </>,
  <>
    {pad(65)}
    {"│"}
  </>,
  <>
    {"  "}
    <Admin>your tailnet</Admin>
    {" ──► "}
    <Admin>brume api (panel)</Admin>
    {` ${"─".repeat(28)}┤ unix sockets`}
  </>,
  <>
    {pad(65)}
    {"▼"}
  </>,
  <>
    {pad(34)}
    {"s3 · state, event log ◄── "}
    <Core>brume engine</Core>
    {"   state, secrets, sessions"}
  </>,
  <>
    {pad(57)}
    {"facts ▲   │ desired state"}
  </>,
  <>
    {pad(63)}
    {"│   ▼"}
  </>,
  <>
    {pad(35)}
    {"s3 · backups, builds ◄── "}
    <Core>brume agent</Core>
    {" ──► docker ──► containers"}
  </>,
  <>
    {pad(55)}
    {"docker socket, listens on nothing"}
  </>,
];

export function ArchitectureDiagram() {
  return (
    <figure className="pixel-border mt-12 bg-terminal">
      <pre
        aria-label="Public traffic goes from the internet through a Cloudflare tunnel and Traefik to your apps, static sites and the webhook. Admin traffic goes from your tailnet to the panel. Both reach the engine over unix sockets. The engine holds the state and the secrets, and sends the desired state to the agent, the only component with the Docker socket."
        className="overflow-x-auto px-6 py-7 font-mono text-[13px] leading-[1.55] text-ink-dim"
      >
        {lines.map((line, index) => (
          // oxlint-disable-next-line react/no-array-index-key -- static lines, never reordered
          <span key={index}>
            {line}
            {"\n"}
          </span>
        ))}
      </pre>
      <figcaption className="border-t-2 border-edge px-6 py-3.5 text-sm text-ink-muted">
        The public path and the admin path never meet. The engine never touches Docker, and the
        agent listens on nothing.
      </figcaption>
    </figure>
  );
}
