import { CheckIcon, CopyIcon } from "lucide-react";
import { useState } from "react";

import type { LogLine } from "~/api/types";
import { Button } from "~/components/ui/button";
import { type JsonTokenKind, jsonTokens, logJson } from "~/helpers/logs";

const TOKEN_COLORS: Record<JsonTokenKind, string> = {
  key: "text-ansi-cyan",
  string: "text-ansi-green",
  number: "text-ansi-yellow",
  literal: "text-ansi-magenta",
  plain: "text-muted-foreground",
};

interface LogDetailsProps {
  readonly line: LogLine;
  readonly serviceName?: string | undefined;
}

// The whole line as JSON, colored token by token. Every token is text: the content of a log
// never becomes markup.
export function LogDetails({ line, serviceName }: LogDetailsProps) {
  const json = logJson(line, serviceName);
  const [copied, setCopied] = useState(false);

  return (
    <div className="relative mx-4 mb-2 rounded-lg border bg-background">
      <Button
        className="absolute top-2 right-2"
        variant="ghost"
        size="icon-sm"
        aria-label="Copier le JSON"
        onClick={() => {
          void navigator.clipboard.writeText(json).then(() => setCopied(true));
        }}
      >
        {copied ? <CheckIcon /> : <CopyIcon />}
      </Button>
      <pre className="overflow-x-auto p-3 pe-12 font-mono text-xs leading-5">
        {jsonTokens(json).map((token, index) => (
          // oxlint-disable-next-line react/no-array-index-key -- a token is its position in the text
          <span key={index} className={TOKEN_COLORS[token.kind]}>
            {token.text}
          </span>
        ))}
      </pre>
    </div>
  );
}
