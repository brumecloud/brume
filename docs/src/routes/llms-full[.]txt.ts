import { createFileRoute } from "@tanstack/react-router";
import { docsLlms, llmsHeader } from "~/lib/source";

export const Route = createFileRoute("/llms-full.txt")({
  server: {
    handlers: {
      GET: async () =>
        new Response(`${llmsHeader}\n${await docsLlms.full()}`, {
          headers: { "Content-Type": "text/plain; charset=utf-8" },
        }),
    },
  },
});
