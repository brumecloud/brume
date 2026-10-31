import { GlobeIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { parseDomain } from "~/helpers/changes";

import { SettingField } from "./setting-field";

interface DomainsSettingProps {
  readonly domains: readonly string[];
  readonly staged: boolean;
  readonly onCommit: (domains: readonly string[]) => void;
}

export function DomainsSetting({ domains, staged, onCommit }: DomainsSettingProps) {
  const [draft, setDraft] = useState("");
  const [invalid, setInvalid] = useState(false);

  return (
    <SettingField
      label="Public domains"
      description="Reachable from the internet over HTTPS, through Cloudflare Tunnel."
      staged={staged}
      error={invalid ? "A domain name, without https:// or a path." : undefined}
    >
      <div className="flex flex-col divide-y rounded-lg border bg-background">
        {domains.map((domain) => (
          <div key={domain} className="flex items-center gap-3 px-4 py-2.5 text-sm">
            <GlobeIcon className="size-4 text-muted-foreground" />
            <span className="flex-1 truncate">{domain}</span>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Remove ${domain}`}
              onClick={() => onCommit(domains.filter((other) => other !== domain))}
            >
              <Trash2Icon />
            </Button>
          </div>
        ))}
        <form
          className="flex gap-2 p-2"
          onSubmit={(event) => {
            event.preventDefault();
            const domain = parseDomain(draft);
            setInvalid(!domain);
            if (domain) {
              onCommit([...domains, domain]);
              setDraft("");
            }
          }}
        >
          <Input
            value={draft}
            aria-label="New domain"
            placeholder="app.example.com"
            onChange={(event) => setDraft(event.target.value)}
          />
          <Button type="submit" variant="outline">
            <PlusIcon />
            Add
          </Button>
        </form>
      </div>
    </SettingField>
  );
}
