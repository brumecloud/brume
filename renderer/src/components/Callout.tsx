import type { PropsWithChildren } from "react";

type CalloutProps = PropsWithChildren<{
  type?: "info" | "warning" | "success" | "danger";
  title?: string;
}>;

export function Callout({
  type = "info",
  title,
  children,
}: CalloutProps) {
  return (
    <aside className={`brume-callout brume-callout-${type}`} data-brume-animate>
      <span aria-hidden="true" className="brume-callout-rail" />
      <div className="brume-callout-content">
        {title ? <strong className="brume-callout-title">{title}</strong> : null}
        <div>{children}</div>
      </div>
    </aside>
  );
}
