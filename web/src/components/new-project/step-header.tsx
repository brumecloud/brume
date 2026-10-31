import { Link } from "@tanstack/react-router";
import { ArrowLeftIcon } from "lucide-react";

import { Button } from "~/components/ui/button";

export function StepHeader({ title }: { readonly title: string }) {
  return (
    <div className="flex items-center gap-2 border-b px-2 py-2">
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Retour"
        render={<Link to="/projects/new" />}
      >
        <ArrowLeftIcon />
      </Button>
      <span className="font-medium text-sm">{title}</span>
    </div>
  );
}
