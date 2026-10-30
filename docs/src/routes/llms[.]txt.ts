import { createFileRoute } from "@tanstack/react-router";
import { llmsIndex } from "~/lib/source";

export const Route = createFileRoute("/llms.txt")({
  server: {
    handlers: {
      GET: () =>
        new Response(llmsIndex(), {
          headers: { "Content-Type": "text/plain; charset=utf-8" },
        }),
    },
  },
});
