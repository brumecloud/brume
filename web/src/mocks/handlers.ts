import { HttpResponse, delay, http } from "msw";

import { environments, projects } from "./seed";

// Enough to see loading states, not enough to get in the way.
const LATENCY_MS = 150;

export const handlers = [
  // The real server rejects any request without this header: so does the mock.
  http.all("/api/*", ({ request }) => {
    if (request.headers.get("X-Brume") !== "1") {
      return HttpResponse.json({ message: "Missing X-Brume header." }, { status: 403 });
    }
    return undefined;
  }),

  http.get("/api/projects", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(projects);
  }),

  http.get("/api/environments", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(environments);
  }),

  // An API route without a mock fails clearly instead of returning index.html.
  http.all("/api/*", () =>
    HttpResponse.json({ message: "No mock for this route yet." }, { status: 501 }),
  ),
];
