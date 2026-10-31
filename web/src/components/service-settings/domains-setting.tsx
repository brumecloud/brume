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

// Reference: infra/screens/04e-service-settings-2-networking.png.
export function DomainsSetting({ domains, staged, onCommit }: DomainsSettingProps) {
  const [draft, setDraft] = useState("");
  const [invalid, setInvalid] = useState(false);

  return (
    <SettingField
      label="Domaines publics"
      description="Joignables sur Internet par Cloudflare Tunnel, en HTTPS."
      staged={staged}
      error={invalid ? "Un nom de domaine, sans https:// ni chemin." : undefined}
    >
      <div className="flex flex-col divide-y rounded-lg border bg-background">
        {domains.map((domain) => (
          <div key={domain} className="flex items-center gap-3 px-4 py-2.5 text-sm">
            <GlobeIcon className="size-4 text-muted-foreground" />
            <span className="flex-1 truncate">{domain}</span>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Retirer ${domain}`}
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
            aria-label="Nouveau domaine"
            placeholder="app.example.com"
            onChange={(event) => setDraft(event.target.value)}
          />
          <Button type="submit" variant="outline">
            <PlusIcon />
            Ajouter
          </Button>
        </form>
      </div>
    </SettingField>
  );
}
