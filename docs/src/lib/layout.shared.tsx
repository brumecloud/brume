import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { repoUrl } from "./shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="flex items-center gap-2">
          <img src="/logo.png" alt="" width={24} height={24} className="size-6 rounded-md" />
          <span className="font-semibold tracking-tight">Brume</span>
          <span className="rounded-full border px-2 py-px text-xs font-medium text-fd-muted-foreground">
            Docs
          </span>
        </span>
      ),
      url: "/docs",
    },
    githubUrl: repoUrl,
  };
}
