import { Link } from "@tanstack/react-router";
import { CloudOffIcon, TriangleAlertIcon } from "lucide-react";

import { Button } from "~/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "~/components/ui/empty";

export function NotFoundPage() {
  return (
    <Empty className="h-dvh">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <CloudOffIcon />
        </EmptyMedia>
        <EmptyTitle>Page introuvable</EmptyTitle>
        <EmptyDescription>Ce projet ou cet environnement n'existe pas, ou plus.</EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button variant="outline" render={<Link to="/projects" />}>
          Retour aux projets
        </Button>
      </EmptyContent>
    </Empty>
  );
}

// An error message can come from a crafted URL: never display it.
export function ErrorPage() {
  return (
    <Empty className="h-dvh">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <TriangleAlertIcon />
        </EmptyMedia>
        <EmptyTitle>Quelque chose s'est mal passé</EmptyTitle>
        <EmptyDescription>
          Rechargez la page. Si l'erreur revient, consultez les logs de Brume.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button variant="outline" render={<Link to="/projects" />}>
          Retour aux projets
        </Button>
      </EmptyContent>
    </Empty>
  );
}
