import { Link } from "@tanstack/react-router";
import { LayoutGridIcon } from "lucide-react";
import type { ReactNode } from "react";

import { BrumeLogo } from "./brume-logo";

// Reference: infra/screens/01-dashboard.png.
export function WorkspaceShell({ children }: { readonly children: ReactNode }) {
  return (
    <div className="flex h-dvh bg-sidebar">
      <aside className="flex w-56 shrink-0 flex-col gap-6 px-3 py-4 max-md:hidden">
        <div className="px-2">
          <BrumeLogo />
        </div>
        <nav className="flex flex-col gap-1">
          <Link
            to="/projects"
            className="flex h-9 items-center gap-3 rounded-lg px-2 text-sidebar-foreground text-sm hover:bg-sidebar-accent data-[status=active]:bg-sidebar-accent data-[status=active]:text-sidebar-accent-foreground"
          >
            <LayoutGridIcon className="size-4" />
            Projets
          </Link>
        </nav>
      </aside>
      <main className="m-2 min-w-0 flex-1 overflow-auto rounded-xl border bg-background md:ml-0">
        {children}
      </main>
    </div>
  );
}
