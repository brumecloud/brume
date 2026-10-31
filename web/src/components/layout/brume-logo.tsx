import { Link } from "@tanstack/react-router";

export function BrumeLogo() {
  return (
    <Link to="/projects" aria-label="Brume, back to projects" className="shrink-0">
      {/* The demo on brume.dev is served under /demo/. */}
      <img src={`${import.meta.env.BASE_URL}logo.png`} alt="" className="size-7 rounded-lg" />
    </Link>
  );
}
