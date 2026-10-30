import { Link } from "@tanstack/react-router";
import { XIcon } from "lucide-react";

import { BrumeLogo } from "~/components/layout/brume-logo";
import { Button } from "~/components/ui/button";
import type { NewProjectStep } from "~/helpers/new-project";

import { STEP_TITLES } from "./new-project-palette";

export function NewProjectHeader({ step }: { readonly step: NewProjectStep | undefined }) {
  return (
    <header className="flex h-14 shrink-0 items-center gap-2 px-4">
      <BrumeLogo />
      <span className="mx-2 h-5 w-px bg-border" />
      <Link to="/projects/new" className="font-medium text-sm">
        Nouveau projet
      </Link>
      {step && (
        <>
          <span className="text-muted-foreground">/</span>
          <span className="text-sm">{STEP_TITLES[step]}</span>
        </>
      )}
      <Button
        className="ms-auto"
        variant="ghost"
        size="icon"
        aria-label="Fermer"
        render={<Link to="/projects" />}
      >
        <XIcon />
      </Button>
    </header>
  );
}
