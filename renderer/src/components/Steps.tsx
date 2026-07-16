import type { PropsWithChildren } from "react";

export function Steps({ children }: PropsWithChildren) {
  return <ol className="brume-steps">{children}</ol>;
}

export function Step({
  title,
  children,
}: PropsWithChildren<{ title: string }>) {
  return (
    <li className="brume-step">
      <h3>{title}</h3>
      <div>{children}</div>
    </li>
  );
}

