import { isValidElement, type ReactElement, type ReactNode } from "react";

type CodeElementProps = {
  className?: string;
  children?: ReactNode;
};

function languageFrom(children: ReactNode): string | undefined {
  if (!isValidElement(children)) return undefined;
  const code = children as ReactElement<CodeElementProps>;
  const language = code.props.className
    ?.split(" ")
    .find((value) => value.startsWith("language-"))
    ?.slice("language-".length);
  return language || undefined;
}

export function CodeBlock({
  children,
  title,
}: {
  children?: ReactNode;
  title?: string;
}) {
  const language = languageFrom(children);
  const label = title || language;

  return (
    <figure className="brume-code-block" data-brume-code-block>
      <div className="brume-code-toolbar">
        <figcaption>{label ?? "Code"}</figcaption>
        <button
          aria-label="Copy code"
          className="brume-copy-code"
          data-brume-copy-code
          type="button"
        >
          Copy
        </button>
      </div>
      <div className="brume-code-viewport" role="region" tabIndex={0}>
        <pre>{children}</pre>
      </div>
    </figure>
  );
}
