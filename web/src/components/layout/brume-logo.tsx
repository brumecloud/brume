import { Link } from "@tanstack/react-router";

export function BrumeLogo() {
  return (
    <Link to="/projects" aria-label="Brume, retour aux projets" className="shrink-0">
      <img src="/logo.png" alt="" className="size-7 rounded-lg" />
    </Link>
  );
}
