import { HttpResponse, delay, http } from "msw";

import type { LogLine } from "~/api/types";

import {
  type Refusal,
  createEnvironment,
  createProject,
  createService,
  listEvents,
  removeSharedVariable,
  revealSecret,
  setSharedVariable,
  variableImpact,
  currentSettings,
  db,
  deleteService,
  deploy,
  deployChanges,
  discardChanges,
  publicChange,
  readNewProjectSource,
  revisionFor,
  stageChange,
} from "./db";
import { backlog, deploymentLogs, liveLine } from "./logs";
import { branchesOf, repositories } from "./seed";

// Enough to see loading states, not enough to get in the way. The demo on brume.dev has to start
// fast: there, a loading state is something to wait for, not something to check.
const LATENCY_MS = import.meta.env.MODE === "demo" ? 40 : 150;

const created = <T extends object>(result: T | Refusal) =>
  "message" in result
    ? HttpResponse.json({ message: result.message }, { status: result.status })
    : HttpResponse.json(result, { status: 201 });

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

  // The mocked server has one admin, always signed in: any token opens a session. The panel only
  // sends one that `parseLoginHash` accepted.
  http.post("/api/session", async ({ request }) => {
    await delay(LATENCY_MS);
    const body: unknown = await request.json().catch(() => undefined);
    const token = typeof body === "object" && body !== null && "token" in body ? body.token : null;
    return typeof token === "string" && token !== ""
      ? new HttpResponse(null, { status: 204 })
      : HttpResponse.json(
          { message: "This login link has expired or was already used." },
          { status: 401 },
        );
  }),

  http.get("/api/session", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json({ admin: "admin" });
  }),

  http.delete("/api/session", async () => {
    await delay(LATENCY_MS);
    return new HttpResponse(null, { status: 204 });
  }),

  http.get("/api/projects", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.projects);
  }),

  http.post("/api/projects", async ({ request }) => {
    await delay(LATENCY_MS * 4);
    const source = readNewProjectSource(await request.json());
    if (!source) {
      return HttpResponse.json({ message: "Invalid project source." }, { status: 400 });
    }
    // This milestone deploys public images only.
    if (source.kind !== "empty" && source.kind !== "image") {
      return HttpResponse.json(
        { message: `${source.kind} sources are not available yet.` },
        { status: 400 },
      );
    }
    return HttpResponse.json(createProject(source), { status: 201 });
  }),

  http.get("/api/environments", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.environments);
  }),

  http.post("/api/projects/:projectId/environments", async ({ params, request }) => {
    await delay(LATENCY_MS * 2);
    return created(createEnvironment(String(params.projectId), await request.json()));
  }),

  http.post("/api/environments/:environmentId/services", async ({ params, request }) => {
    await delay(LATENCY_MS * 4);
    return created(createService(String(params.environmentId), await request.json()));
  }),

  http.get("/api/services", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.services);
  }),

  http.get("/api/github/repositories", async () => {
    await delay(LATENCY_MS * 2);
    return HttpResponse.json(repositories);
  }),

  http.get("/api/github/repositories/:owner/:name/branches", async ({ params }) => {
    await delay(LATENCY_MS);
    const fullName = `${String(params.owner)}/${String(params.name)}`;
    if (!repositories.some((repository) => repository.fullName === fullName)) {
      return HttpResponse.json({ message: "Repository not found." }, { status: 404 });
    }
    return HttpResponse.json(branchesOf(fullName));
  }),

  http.get("/api/services/:serviceId/settings", async ({ params }) => {
    await delay(LATENCY_MS);
    const service = db.services.find(({ id }) => id === params.serviceId);
    return service
      ? HttpResponse.json(currentSettings(service))
      : HttpResponse.json({ message: "Service not found." }, { status: 404 });
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
      return HttpResponse.json({ message: "No deployment to redeploy." }, { status: 409 });
    }
    return HttpResponse.json(deploy(active, "admin"), { status: 201 });
  }),

  http.post("/api/deployments/:deploymentId/rollback", async ({ params }) => {
    await delay(LATENCY_MS * 4);
    const target = db.deployments.find(({ id }) => id === params.deploymentId);
    if (!target?.rollbackable) {
      return HttpResponse.json(
        { message: "This deployment's artifact no longer exists." },
        { status: 409 },
      );
    }
    return HttpResponse.json(deploy(target, "admin"), { status: 201 });
  }),

  http.get("/api/variables", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.variables);
  }),

  http.get("/api/variables/impact", async ({ request }) => {
    await delay(LATENCY_MS);
    const impact = variableImpact(new URL(request.url).searchParams);
    return "message" in impact
      ? HttpResponse.json({ message: impact.message }, { status: impact.status })
      : HttpResponse.json(impact);
  }),

  http.post("/api/variables", async ({ request }) => {
    await delay(LATENCY_MS * 2);
    return created(setSharedVariable(await request.json()));
  }),

  http.delete("/api/variables/:variableId", async ({ params }) => {
    await delay(LATENCY_MS * 2);
    const removal = removeSharedVariable(String(params.variableId));
    return "message" in removal
      ? HttpResponse.json({ message: removal.message }, { status: removal.status })
      : HttpResponse.json(removal);
  }),

  http.get("/api/changes", async () => {
    await delay(LATENCY_MS);
    return HttpResponse.json(db.changes.map(publicChange));
  }),

  http.post("/api/changes", async ({ request }) => {
    await delay(LATENCY_MS);
    const change = stageChange(await request.json());
    return typeof change === "string"
      ? HttpResponse.json({ message: change }, { status: 400 })
      : HttpResponse.json(change, { status: 201 });
  }),

  http.delete("/api/changes/:changeId", async ({ params }) => {
    await delay(LATENCY_MS);
    db.changes = db.changes.filter(({ id }) => id !== params.changeId);
    return new HttpResponse(null, { status: 204 });
  }),

  http.post("/api/environments/:environmentId/changes/deploy", async ({ params }) => {
    await delay(LATENCY_MS * 4);
    return HttpResponse.json({ deployed: deployChanges(String(params.environmentId)) });
  }),

  http.delete("/api/environments/:environmentId/changes", async ({ params }) => {
    await delay(LATENCY_MS);
    discardChanges(String(params.environmentId));
    return new HttpResponse(null, { status: 204 });
  }),

  // A reveal is audited by the real server.
  http.post("/api/variables/:variableId/reveal", async ({ params }) => {
    await delay(LATENCY_MS);
    const value = revealSecret(String(params.variableId));
    return value === undefined
      ? HttpResponse.json({ message: "Secret not found." }, { status: 404 })
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
      : HttpResponse.json({ message: "Deployment not found." }, { status: 404 });
  }),

  http.get("/api/deployments/:deploymentId/logs", async ({ params, request }) => {
    await delay(LATENCY_MS * 2);
    const deployment = db.deployments.find(({ id }) => id === params.deploymentId);
    const service = db.services.find(({ id }) => id === deployment?.serviceId);
    const stream =
      new URL(request.url).searchParams.get("stream") === "deploy" ? "deploy" : "build";
    return deployment && service
      ? HttpResponse.json(deploymentLogs(deployment, service, stream))
      : HttpResponse.json({ message: "Deployment not found." }, { status: 404 });
  }),

  http.get("/api/events", async ({ request }) => {
    await delay(LATENCY_MS);
    const page = listEvents(new URL(request.url).searchParams);
    return "message" in page
      ? HttpResponse.json({ message: page.message }, { status: page.status })
      : HttpResponse.json(page);
  }),

  // An API route without a mock fails clearly instead of returning index.html.
  http.all("/api/*", () =>
    HttpResponse.json({ message: "No mock for this route yet." }, { status: 501 }),
  ),
];
