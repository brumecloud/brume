import type { PropsWithChildren, ReactNode } from "react";

export function Decision({
  status = "proposed",
  children,
}: PropsWithChildren<{ status?: "proposed" | "accepted" | "rejected" }>) {
  return (
    <aside className={`brume-decision brume-decision-${status}`}>
      <strong>Decision: {status}</strong>
      <div>{children}</div>
    </aside>
  );
}

export function Risk({
  severity = "medium",
  children,
}: PropsWithChildren<{ severity?: "low" | "medium" | "high" }>) {
  return (
    <aside className={`brume-risk brume-risk-${severity}`}>
      <strong>Risk: {severity}</strong>
      <div>{children}</div>
    </aside>
  );
}

export function FileTree({ children }: PropsWithChildren) {
  return <pre className="brume-file-tree">{children}</pre>;
}

export function Card({
  title,
  href,
  children,
}: PropsWithChildren<{ title: string; href?: string }>) {
  const content: ReactNode = (
    <>
      <strong>{title}</strong>
      <div>{children}</div>
    </>
  );
  return href ? (
    <a className="brume-card" href={href}>
      {content}
    </a>
  ) : (
    <article className="brume-card">{content}</article>
  );
}

export function CardGrid({ children }: PropsWithChildren) {
  return <div className="brume-card-grid">{children}</div>;
}

export function CodeGroup({ children }: PropsWithChildren) {
  return <div className="brume-code-group">{children}</div>;
}

export function Mermaid({ children }: PropsWithChildren) {
  return <div className="mermaid">{children}</div>;
}

