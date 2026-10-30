import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { createServerFn } from "@tanstack/react-start";
import { staticFunctionMiddleware } from "@tanstack/start-static-server-functions";
import { useFumadocsLoader } from "fumadocs-core/source/client";
import { Callout } from "fumadocs-ui/components/callout";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
  MarkdownCopyButton,
  ViewOptionsPopover,
} from "fumadocs-ui/layouts/docs/page";
import { Suspense, use } from "react";
import { useMDXComponents } from "~/components/mdx";
import { baseOptions } from "~/lib/layout.shared";
import { getPageMarkdownUrl, gitConfig, repoUrl, siteUrl } from "~/lib/shared";
import { docs, source } from "~/lib/source";

export const Route = createFileRoute("/docs/$")({
  component: Page,
  loader: async ({ params }) => {
    const slugs = params._splat?.split("/") ?? [];
    const data = await loader({ data: slugs });
    await docs.getPage(data.path)?.preload();
    return data;
  },
  head: ({ loaderData }) => ({
    meta: loaderData
      ? [
          { title: `${loaderData.title} | Brume Docs` },
          ...(loaderData.description
            ? [{ name: "description", content: loaderData.description }]
            : []),
        ]
      : [],
  }),
});

const loader = createServerFn({ method: "GET" })
  .validator((slugs: string[]) => slugs)
  .middleware([staticFunctionMiddleware])
  .handler(async ({ data: slugs }) => {
    const page = source.getPage(slugs);
    if (!page) throw notFound();

    return {
      path: page.path,
      title: page.data.title,
      description: page.data.description,
      isLanding: page.slugs.length === 0,
      markdownUrl: getPageMarkdownUrl(page).url,
      pageUrl: `${siteUrl}${page.url}`,
      pageTree: await source.serializePageTree(source.getPageTree()),
    };
  });

interface ContentProps {
  path: string;
  isLanding: boolean;
  markdownUrl: string;
  pageUrl: string;
}

function Content({ path, isLanding, markdownUrl, pageUrl }: ContentProps) {
  const page = docs.getPage(path);
  if (!page) throw new Error(`unknown page: ${path}`);

  const { toc } = use(page.load());
  const MDX = page.body;

  // The docs home is a landing page: its MDX draws its own header.
  if (isLanding) {
    return (
      <DocsPage full breadcrumb={{ enabled: false }} footer={{ enabled: false }}>
        <DocsBody>
          <MDX components={useMDXComponents()} />
        </DocsBody>
      </DocsPage>
    );
  }

  const githubUrl = `${repoUrl}/blob/${gitConfig.branch}/${gitConfig.contentDir}/${path}`;

  return (
    <DocsPage toc={toc} full={page.full} tableOfContent={{ style: "clerk" }}>
      <DocsTitle>{page.title}</DocsTitle>
      <DocsDescription>{page.description}</DocsDescription>
      <div className="-mt-4 flex flex-row items-center gap-2 border-b pb-6">
        <MarkdownCopyButton markdownUrl={markdownUrl} />
        <ViewOptionsPopover markdownUrl={markdownUrl} githubUrl={githubUrl} pageUrl={pageUrl} />
      </div>
      <DocsBody>
        {page.status === "planned" && (
          <Callout type="warn" title="Planned">
            This page describes how Brume alpha 0.1 is designed to work. It is not built yet.
          </Callout>
        )}
        <MDX components={useMDXComponents()} />
      </DocsBody>
    </DocsPage>
  );
}

function Page() {
  const { pageTree, path, isLanding, markdownUrl, pageUrl } = useFumadocsLoader(
    Route.useLoaderData(),
  );

  return (
    <DocsLayout {...baseOptions()} tree={pageTree}>
      {/* Lets the prerender crawler find the Markdown twin of the page. */}
      <Link to={markdownUrl} hidden />
      <Suspense>
        <Content path={path} isLanding={isLanding} markdownUrl={markdownUrl} pageUrl={pageUrl} />
      </Suspense>
    </DocsLayout>
  );
}
