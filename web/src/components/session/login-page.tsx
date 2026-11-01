import { KeyRoundIcon } from "lucide-react";

import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";

// No password form: the only way in is the one-time link printed by `brume dev`.
export function LoginPage({ refused }: { readonly refused: boolean }) {
  return (
    <Empty className="h-dvh">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <KeyRoundIcon />
        </EmptyMedia>
        <EmptyTitle>{refused ? "This login link no longer works" : "Sign in to Brume"}</EmptyTitle>
        {refused && <EmptyDescription>The link has expired or was already used.</EmptyDescription>}
        <EmptyDescription>
          Open the login link that <code>sudo brume admin login</code> prints on the server, or that{" "}
          <code>brume dev</code> prints on your machine.
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
