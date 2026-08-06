import { useEffect, useRef, useState } from "react";
import { Check, Copy, Link as LinkIcon, LogIn, Wand2 } from "lucide-react";
import {
  ApiRequestError,
  type AuthMode,
  type ToolbarAccessState,
  type ToolbarApi,
} from "./api";
import { ACCESS_COPY, ACCESS_MODES } from "./access-copy";
import { cn } from "@/overlay/lib/utils";
import { Button } from "@/overlay/ui/button";
import { Input } from "@/overlay/ui/input";
import { Separator } from "@/overlay/ui/separator";
import { Spinner } from "@/overlay/ui/spinner";

interface ManagementViewProps {
  api: ToolbarApi;
  active: boolean;
  onAuthModeChange: (mode: AuthMode) => void;
}

type Phase = "loading" | "signin" | "ready" | "error";

const PASSWORD_ALPHABET =
  "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%";

function generatePassword(): string {
  const values = crypto.getRandomValues(new Uint32Array(20));
  return Array.from(values, (value) => PASSWORD_ALPHABET[value % PASSWORD_ALPHABET.length]).join(
    "",
  );
}

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <p className="m-0 px-1 text-[11px] font-semibold tracking-[0.08em] text-muted-foreground uppercase">
      {children}
    </p>
  );
}

export function ManagementView({ api, active, onAuthModeChange }: ManagementViewProps) {
  const [phase, setPhase] = useState<Phase>("loading");
  const [access, setAccess] = useState<ToolbarAccessState | null>(null);
  const [selectedMode, setSelectedMode] = useState<AuthMode>("token");
  const [password, setPassword] = useState("");
  const [revealPassword, setRevealPassword] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [inviteUrl, setInviteUrl] = useState("");
  const [inviteBusy, setInviteBusy] = useState(false);
  const [revoking, setRevoking] = useState<string | null>(null);
  const requested = useRef(false);

  const load = () => {
    setPhase("loading");
    setError("");
    api
      .access()
      .then((state) => {
        setAccess(state);
        setSelectedMode(state.auth_mode);
        onAuthModeChange(state.auth_mode);
        setPhase("ready");
      })
      .catch((cause) => {
        if (cause instanceof ApiRequestError && cause.status === 401) {
          setPhase("signin");
        } else {
          setPhase("error");
        }
      });
  };

  useEffect(() => {
    if (!active || requested.current) return;
    requested.current = true;
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active]);

  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (
        event.origin !== location.origin ||
        (event.data as { type?: string } | null)?.type !== "brume-owner-authenticated"
      ) {
        return;
      }
      load();
    };
    addEventListener("message", onMessage);
    return () => removeEventListener("message", onMessage);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const signIn = () => {
    const returnTo = new URL(location.href);
    returnTo.searchParams.set("_brume_auth_complete", "1");
    const url =
      api.authOrigin +
      "/auth/github/start?site=" +
      encodeURIComponent(api.site) +
      "&site_return_to=" +
      encodeURIComponent(returnTo.toString());
    open(url, "brume-auth");
  };

  const needsPassword =
    selectedMode === "password" && !(access?.has_password && access.auth_mode === "password");
  const dirty = access !== null && (selectedMode !== access.auth_mode || password.length > 0);

  const save = () => {
    if (!access || saving) return;
    if (needsPassword && password.length === 0) {
      setError("Choose a password first");
      return;
    }
    if (password.length > 0 && (password.length < 8 || password.length > 256)) {
      setError("Passwords must contain between 8 and 256 characters");
      return;
    }
    setSaving(true);
    setError("");
    setNotice("");
    api
      .updateSettings(selectedMode, password.length > 0 ? password : undefined)
      .then((state) => {
        setAccess(state);
        setSelectedMode(state.auth_mode);
        setPassword("");
        setRevealPassword(false);
        setInviteUrl("");
        onAuthModeChange(state.auth_mode);
        setNotice("Access updated");
      })
      .catch((cause) => setError(cause instanceof Error ? cause.message : "Could not save"))
      .finally(() => setSaving(false));
  };

  const invite = () => {
    if (inviteBusy) return;
    setInviteBusy(true);
    setError("");
    setNotice("");
    api
      .createInvitation(location.href)
      .then(async (response) => {
        setInviteUrl(response.url);
        try {
          await navigator.clipboard.writeText(response.url);
          setNotice("Invite link copied. It expires in 24 hours.");
        } catch {
          setNotice("Invite link created. It expires in 24 hours.");
        }
      })
      .catch((cause) =>
        setError(cause instanceof Error ? cause.message : "Could not create the invite link"),
      )
      .finally(() => setInviteBusy(false));
  };

  const copyInvite = () =>
    navigator.clipboard
      .writeText(inviteUrl)
      .then(() => setNotice("Invite link copied"))
      .catch(() => setError("Could not copy the invite link"));

  const revoke = (publicId: string) => {
    if (revoking) return;
    if (!confirm("Revoke access for this recipient? Their token stops working immediately.")) {
      return;
    }
    setRevoking(publicId);
    setError("");
    setNotice("");
    api
      .revokeGrant(publicId)
      .then((response) => {
        setAccess((current) => (current ? { ...current, grants: response.grants } : current));
        setNotice("Access revoked");
      })
      .catch((cause) => setError(cause instanceof Error ? cause.message : "Could not revoke"))
      .finally(() => setRevoking(null));
  };

  if (phase === "loading") {
    return (
      <div className="flex items-center justify-center py-10">
        <Spinner className="text-muted-foreground" />
      </div>
    );
  }

  if (phase === "signin") {
    return (
      <div className="flex flex-col gap-2.5 px-1 py-2">
        <p className="m-0 text-sm text-muted-foreground">
          Sign in with the GitHub account that deployed this website to manage its access.
        </p>
        <Button type="button" className="w-full" onClick={signIn}>
          <LogIn data-icon="inline-start" aria-hidden />
          Sign in with GitHub
        </Button>
      </div>
    );
  }

  if (phase === "error" || !access) {
    return (
      <div className="flex flex-col gap-2.5 px-1 py-2">
        <p className="m-0 text-sm text-muted-foreground">Could not load the access settings.</p>
        <Button type="button" variant="secondary" className="w-full" onClick={load}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2.5">
      <SectionTitle>Visibility</SectionTitle>
      <div className="flex flex-col gap-1" role="radiogroup" aria-label="Website visibility">
        {ACCESS_MODES.map((mode) => {
          const copy = ACCESS_COPY[mode];
          const Icon = copy.icon;
          const selected = selectedMode === mode;
          return (
            <button
              key={mode}
              type="button"
              role="radio"
              aria-checked={selected}
              className={cn(
                "flex cursor-pointer items-center gap-2.5 rounded-lg border border-transparent p-2 text-left outline-none transition-colors select-none",
                "focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
                selected ? "border-border bg-muted" : "hover:bg-muted/50",
              )}
              onClick={() => {
                setSelectedMode(mode);
                setError("");
                setNotice("");
              }}
            >
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-accent text-foreground">
                <Icon className="size-4" aria-hidden />
              </span>
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="text-sm leading-tight font-medium">{copy.label}</span>
                <span className="truncate text-xs leading-tight text-muted-foreground">
                  {copy.description}
                </span>
              </span>
              {selected && <Check className="size-4 shrink-0 text-foreground" aria-hidden />}
            </button>
          );
        })}
      </div>
      {selectedMode === "password" && (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
          <Input
            type={revealPassword ? "text" : "password"}
            autoComplete="new-password"
            minLength={8}
            maxLength={256}
            placeholder={
              access.has_password && access.auth_mode === "password"
                ? "Leave empty to keep current password"
                : "New password"
            }
            value={password}
            className="bg-input/20"
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
          <Button
            type="button"
            variant="secondary"
            aria-label="Generate password"
            onClick={() => {
              setPassword(generatePassword());
              setRevealPassword(true);
            }}
          >
            <Wand2 data-icon="inline-start" aria-hidden />
            Generate
          </Button>
        </div>
      )}
      <Button type="button" className="w-full" disabled={!dirty || saving} onClick={save}>
        {saving && <Spinner data-icon="inline-start" aria-hidden />}
        Save
      </Button>
      {access.auth_mode === "token" && (
        <>
          <Separator className="my-1" />
          <SectionTitle>Invite link</SectionTitle>
          {inviteUrl ? (
            <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <Input readOnly value={inviteUrl} className="bg-input/20" onFocus={(event) => event.currentTarget.select()} />
              <Button type="button" variant="secondary" size="icon" aria-label="Copy invite link" onClick={copyInvite}>
                <Copy aria-hidden />
              </Button>
            </div>
          ) : (
            <Button
              type="button"
              variant="secondary"
              className="w-full"
              disabled={inviteBusy}
              onClick={invite}
            >
              {inviteBusy ? (
                <Spinner data-icon="inline-start" aria-hidden />
              ) : (
                <LinkIcon data-icon="inline-start" aria-hidden />
              )}
              Create invite link
            </Button>
          )}
          <p className="m-0 px-1 text-xs text-muted-foreground">
            Each link works once and expires after 24 hours.
          </p>
          <Separator className="my-1" />
          <SectionTitle>People with access</SectionTitle>
          {access.grants.length === 0 ? (
            <p className="m-0 px-1 text-sm text-muted-foreground">No one has been invited yet.</p>
          ) : (
            <div className="flex flex-col gap-1">
              {access.grants.map((grant) => (
                <div
                  key={grant.public_id}
                  className="flex items-center gap-2.5 rounded-lg p-2 hover:bg-muted/50"
                >
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate text-sm leading-tight font-medium">
                      {grant.display_name ?? grant.public_id}
                    </span>
                    <span className="truncate text-xs leading-tight text-muted-foreground">
                      {grant.display_name ? grant.public_id + " · " : ""}
                      Invited {new Date(grant.granted_at).toLocaleDateString()}
                    </span>
                  </span>
                  <Button
                    type="button"
                    variant="destructive"
                    size="sm"
                    disabled={revoking !== null}
                    onClick={() => revoke(grant.public_id)}
                  >
                    {revoking === grant.public_id ? <Spinner aria-hidden /> : "Revoke"}
                  </Button>
                </div>
              ))}
            </div>
          )}
        </>
      )}
      <div aria-live="polite">
        {notice && <p className="m-0 px-1 text-xs text-success">{notice}</p>}
        {error && <p className="m-0 px-1 text-xs text-destructive">{error}</p>}
      </div>
    </div>
  );
}
