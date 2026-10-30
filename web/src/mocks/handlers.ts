import { HttpResponse, delay, http } from "msw";

import type { LogLine } from "~/api/types";

import {
  createProject,
  createVariable,
  db,
  deleteService,
  deploy,
  readNewProjectSource,
  revisionFor,
  settingsFor,
} from "./db";
import { backlog, deploymentLogs, liveLine } from "./logs";
import { repositories } from "./seed";

// Enough to see loading states, not enough to get in the way.
const LATENCY_MS = 150;

export const handlers = [
  // The real server rejects any request without this header: so does the mock. EventSource
  // cannot send it: the log stream is guarded by Fetch Metadata instead.
  http.all("/api/*", ({ request }) => {
    const stream = request.headers.get("Accept") === "text/event-stream";
    if (!stream && request.headers.get("X-Brume") !== "1") {
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

  http.get("/api/services/:serviceId/settings", async ({ params }) => {
    await delay(LATENCY_MS);
    const service = db.services.find(({ id }) => id === params.serviceId);
    return service
      ? HttpResponse.json(settingsFor(service))
      : HttpResponse.json({ message: "Service introuvable." }, { status: 404 });
  }),

  http.delete("/api/services/:serviceId", async ({ params }) => {
    await delay(LATENCY_MS * 2);
    deleteService(String(params.serviceId));
    return new HttpResponse(null, { status: 204 });
  }),

  http.get("/api/deployments", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.deployments);
  }),

  // Redeploys the revision in service, with the current value of its secrets.
  http.post("/api/services/:serviceId/redeploy", async ({ params }) => {
    await delay(LATENCY_MS * 4);
    const active = db.deployments.find(
      (deployment) => deployment.serviceId === params.serviceId && deployment.status === "active",
    );
    if (!active) {
      return HttpResponse.json({ message: "Aucun déploiement à relancer." }, { status: 409 });
    }
    return HttpResponse.json(deploy(active, "admin"), { status: 201 });
  }),

  http.post("/api/deployments/:deploymentId/rollback", async ({ params }) => {
    await delay(LATENCY_MS * 4);
    const target = db.deployments.find(({ id }) => id === params.deploymentId);
    if (!target?.rollbackable) {
      return HttpResponse.json(
        { message: "L'artefact de ce déploiement n'existe plus." },
        { status: 409 },
      );
    }
    return HttpResponse.json(deploy(target, "admin"), { status: 201 });
  }),

  http.get("/api/variables", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.variables);
  }),

  http.post("/api/variables", async ({ request }) => {
    await delay(LATENCY_MS * 2);
    const created = createVariable(await request.json());
    return typeof created === "string"
      ? HttpResponse.json({ message: created }, { status: 409 })
      : HttpResponse.json(created, { status: 201 });
  }),

  http.delete("/api/variables/:variableId", async ({ params }) => {
    await delay(LATENCY_MS * 2);
    db.variables = db.variables.filter(({ id }) => id !== params.variableId);
    return new HttpResponse(null, { status: 204 });
  }),

  // A reveal is audited by the real server.
  http.post("/api/variables/:variableId/reveal", async ({ params }) => {
    await delay(LATENCY_MS);
    const value = db.secretValues.get(String(params.variableId));
    return value === undefined
      ? HttpResponse.json({ message: "Secret introuvable." }, { status: 404 })
      : HttpResponse.json({ value });
  }),

  // A backlog of the last 15 minutes, then a few lines per second. The stream is built by hand:
  // its `cancel` tells when the browser closed the EventSource.
  http.get("/api/environments/:environmentId/logs", ({ params }) => {
    const services = db.services.filter(
      ({ environmentId }) => environmentId === params.environmentId,
    );
    const encoder = new TextEncoder();
    const frame = (lines: readonly LogLine[]) =>
      encoder.encode(`data: ${JSON.stringify(lines)}\n\n`);
    let timer: ReturnType<typeof setInterval> | undefined;
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(frame(backlog(services, 200, 15 * 60_000)));
        timer = setInterval(() => {
          const line = liveLine(services);
          try {
            if (line) {
              controller.enqueue(frame([line]));
            }
          } catch {
            clearInterval(timer);
          }
        }, 700);
      },
      cancel() {
        clearInterval(timer);
      },
    });
    return new HttpResponse(stream, { headers: { "Content-Type": "text/event-stream" } });
  }),

  http.get("/api/deployments/:deploymentId/revision", async ({ params }) => {
    await delay(LATENCY_MS);
    const deployment = db.deployments.find(({ id }) => id === params.deploymentId);
    const revision = deployment && revisionFor(deployment);
    return revision
      ? HttpResponse.json(revision)
      : HttpResponse.json({ message: "Déploiement introuvable." }, { status: 404 });
  }),

  http.get("/api/deployments/:deploymentId/logs", async ({ params, request }) => {
    await delay(LATENCY_MS * 2);
    const deployment = db.deployments.find(({ id }) => id === params.deploymentId);
    const service = db.services.find(({ id }) => id === deployment?.serviceId);
    const stream =
      new URL(request.url).searchParams.get("stream") === "deploy" ? "deploy" : "build";
    return deployment && service
      ? HttpResponse.json(deploymentLogs(deployment, service, stream))
      : HttpResponse.json({ message: "Déploiement introuvable." }, { status: 404 });
  }),

  // An API route without a mock fails clearly instead of returning index.html.
  http.all("/api/*", () =>
    HttpResponse.json({ message: "No mock for this route yet." }, { status: 501 }),
  ),
];
