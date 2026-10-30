import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { Banner } from "fumadocs-ui/components/banner";
import { RootProvider } from "fumadocs-ui/provider/tanstack";
import SearchDialog from "~/components/search";
import appCss from "~/styles/app.css?url";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "Brume Docs" },
      {
        name: "description",
        content: "Documentation of Brume, the open source and self-hostable alternative to Railway.",
      },
    ],
    links: [
      { rel: "stylesheet", href: appCss },
      { rel: "icon", type: "image/png", href: "/logo.png" },
    ],
  }),
  component: RootComponent,
});

function RootComponent() {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body className="flex min-h-screen flex-col">
        <RootProvider search={{ SearchDialog }}>
          <Banner>
            Brume is in design. Nothing described here is usable yet.
          </Banner>
          <Outlet />
        </RootProvider>
        <Scripts />
      </body>
    </html>
  );
}
