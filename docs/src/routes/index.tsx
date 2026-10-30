import { createFileRoute, Link } from "@tanstack/react-router";
import { HomeLayout } from "fumadocs-ui/layouts/home";
import { baseOptions } from "~/lib/layout.shared";

export const Route = createFileRoute("/")({
  component: Home,
});

function Home() {
  return (
    <HomeLayout {...baseOptions()}>
      <div className="flex flex-1 flex-col items-center justify-center gap-6 px-4 py-16 text-center">
        <img src="/logo.png" alt="" width={72} height={72} className="size-18 rounded-2xl" />
        <h1 className="text-4xl font-semibold tracking-tight">Brume</h1>
        <p className="max-w-xl text-lg text-fd-muted-foreground">
          An open source, self-hostable alternative to Railway, Coolify and Dokploy, with a simpler
          UX and developer experience.
        </p>
        <Link
          to="/docs/$"
          params={{ _splat: "" }}
          className="rounded-lg bg-fd-primary px-4 py-2 text-sm font-medium text-fd-primary-foreground"
        >
          Read the docs
        </Link>
      </div>
    </HomeLayout>
  );
}
