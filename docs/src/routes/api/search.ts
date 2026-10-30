import { createFileRoute } from "@tanstack/react-router";
import { createFromSource } from "fumadocs-core/search/server";
import { source } from "~/lib/source";

const server = createFromSource(source, {
  language: "english",
});

/** Prerendered to a static file: the browser downloads the index and searches it locally. */
export const Route = createFileRoute("/api/search")({
  server: {
    handlers: {
      GET: () => server.staticGET(),
    },
  },
});
