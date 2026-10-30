import { HttpResponse, delay, http } from "msw";

import { createProject, db, readNewProjectSource } from "./db";
import { repositories } from "./seed";

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
    return HttpResponse.json(db.projects);
  }),

  http.post("/api/projects", async ({ request }) => {
    await delay(LATENCY_MS * 4);
    const source = readNewProjectSource(await request.json());
    if (!source) {
      return HttpResponse.json({ message: "Source de projet invalide." }, { status: 400 });
    }
    return HttpResponse.json(createProject(source), { status: 201 });
  }),

  http.get("/api/environments", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.environments);
  }),

  http.get("/api/services", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.services);
  }),

  http.get("/api/github/repositories", async () => {
    await delay(LATENCY_MS * 2);
    return HttpResponse.json(repositories);
  }),

  // An API route without a mock fails clearly instead of returning index.html.
  http.all("/api/*", () =>
    HttpResponse.json({ message: "No mock for this route yet." }, { status: 501 }),
  ),
];
