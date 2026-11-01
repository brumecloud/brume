import { useSuspenseQuery } from "@tanstack/react-query";
import { GlobeIcon } from "lucide-react";
import { useState } from "react";

import { ingressQuery } from "~/api/ingress";
import type { SettingValue } from "~/api/types";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
} from "~/components/ui/input-group";
import { validateSetting } from "~/helpers/changes";
import { publicHost } from "~/helpers/ingress";
import { domainUrl } from "~/helpers/services";

import { SettingField } from "./setting-field";

interface DomainSettingProps {
  // The subdomain after the staged change, empty for none. Give the component this value as `key`
  // too: a deploy or a discard then resets what was typed.
  readonly value: string;
  // The hosts that the active deployment serves.
  readonly domains: readonly string[];
  readonly staged: boolean;
  readonly onCommit: (value: SettingValue) => void;
}

// A subdomain of the server's zone, staged when the field loses focus or on Enter, once valid.
// The server has no zone until `brume install --domain` sets one up.
export function DomainSetting({ value, domains, staged, onCommit }: DomainSettingProps) {
  const { data: ingress } = useSuspenseQuery(ingressQuery);
  const [text, setText] = useState(value);
  const [error, setError] = useState<string>();
  const host = ingress ? publicHost(domains, ingress.zone) : undefined;

  const commit = () => {
    const result = validateSetting("domain", text);
    setError("error" in result ? result.error : undefined);
    if ("value" in result) {
      onCommit(result.value);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <SettingField
        label="Public domain"
        description={
          ingress ? (
            "Reachable from the internet over HTTPS, through Cloudflare Tunnel."
          ) : (
            <>
              Run <code>brume install --domain</code> on the server to enable it.
            </>
          )
        }
        staged={staged}
        error={error}
      >
        <InputGroup>
          <InputGroupInput
            value={text}
            disabled={!ingress}
            aria-label="Public domain"
            aria-invalid={error ? true : undefined}
            placeholder="app"
            onChange={(event) => setText(event.target.value)}
            onBlur={commit}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                commit();
              }
            }}
          />
          {ingress && (
            <InputGroupAddon align="inline-end">
              <InputGroupText>{`.${ingress.zone}`}</InputGroupText>
            </InputGroupAddon>
          )}
        </InputGroup>
      </SettingField>
      {host && (
        <a
          href={domainUrl(host)}
          target="_blank"
          rel="noreferrer"
          className="flex w-fit items-center gap-2 text-sm hover:underline"
        >
          <GlobeIcon className="size-4 text-muted-foreground" />
          {`https://${host}`}
        </a>
      )}
    </div>
  );
}
