import type {
  Environment,
  EnvironmentId,
  Project,
  ProjectId,
  Repository,
  Service,
  ServiceId,
  ServiceKind,
  ServiceStatus,
} from "~/api/types";

// The demo dataset: five made-up businesses running the kind of apps teams deploy every day. The
// same data feeds the dev server, the browser tests and the console embedded on brume.dev. Their
// domains were unregistered when picked: the console links to them.

// Dates follow the page load, so the demo always looks in use.
const NOW = Date.now();
const MINUTE_MS = 60_000;

export const minutesAgo = (minutes: number) => new Date(NOW - minutes * MINUTE_MS).toISOString();

const HOUR = 60;
const DAY = 24 * HOUR;

const project = (id: string, name: string, description: string, updatedAt: string): Project => ({
  id: id as ProjectId,
  name,
  description,
  updatedAt,
});

const environment = (projectId: ProjectId, name: string): Environment => ({
  id: `${projectId}-${name}` as EnvironmentId,
  projectId,
  name,
});

interface ServiceOptions {
  readonly status?: ServiceStatus;
  readonly domains?: readonly string[];
  readonly volume?: string;
  readonly mountPath?: string;
  // Names of the services of the same environment this one connects to.
  readonly dependsOn?: readonly string[];
}

const MOUNT_PATHS: Partial<Record<ServiceKind, string>> = {
  postgres: "/var/lib/postgresql/data",
  mongodb: "/data/db",
  redis: "/data",
};

const service = (
  env: Environment,
  name: string,
  kind: ServiceKind,
  { status = "online", domains = [], volume, mountPath, dependsOn = [] }: ServiceOptions = {},
): Service => ({
  id: `${env.id}-${name}` as ServiceId,
  environmentId: env.id,
  name,
  slug: name.toLowerCase(),
  kind,
  status,
  domains,
  volume: volume ? { name: volume, mountPath: mountPath ?? MOUNT_PATHS[kind] ?? "/data" } : null,
  dependencies: dependsOn.map((dependency) => `${env.id}-${dependency}` as ServiceId),
});

export const northbean = project(
  "c0a1d9f0-fd9b-4683-b608-2bf60613b499",
  "northbean",
  "Online store for a coffee roaster: catalog, checkout and coffee subscriptions.",
  minutesAgo(12),
);
export const routelane = project(
  "5b0e7c52-8d0b-4a3e-9a51-3c1f0e8e2d10",
  "routelane",
  "Fleet tracking for carriers: live GPS positions, delivery routes and alerts.",
  minutesAgo(2 * HOUR),
);
const deskmind = project(
  "9f3c2a71-64b5-4f0e-b2d8-0a7e5c1d4b22",
  "deskmind",
  "AI support assistant: answers tickets from the product documentation.",
  minutesAgo(DAY + 3 * HOUR),
);
const tallybook = project(
  "2d7e9b14-0c3a-4e61-8f5d-6b2a1c9e7f33",
  "tallybook",
  "Quotes and invoices for small businesses: PDFs, automatic reminders and accounting export.",
  minutesAgo(3 * DAY),
);
const clinicslot = project(
  "7e4b1c38-2f9d-4a06-b5e3-8d1a6c0f9b44",
  "clinicslot",
  "Appointment booking for medical practices: shared calendar and SMS reminders.",
  minutesAgo(9 * DAY),
);

export const projects: readonly Project[] = [northbean, routelane, deskmind, tallybook, clinicslot];

const northbeanProduction = environment(northbean.id, "production");
const northbeanStaging = environment(northbean.id, "staging");
const routelaneProduction = environment(routelane.id, "production");
const deskmindProduction = environment(deskmind.id, "production");
const tallybookProduction = environment(tallybook.id, "production");
const tallybookStaging = environment(tallybook.id, "staging");
const clinicslotProduction = environment(clinicslot.id, "production");

export const environments: readonly Environment[] = [
  northbeanProduction,
  northbeanStaging,
  routelaneProduction,
  deskmindProduction,
  tallybookProduction,
  tallybookStaging,
  clinicslotProduction,
];

export const services: readonly Service[] = [
  service(northbeanProduction, "storefront", "static", { domains: ["northbean.coffee"] }),
  service(northbeanProduction, "shop-api", "github", {
    domains: ["api.northbean.coffee"],
    dependsOn: ["PostgreSQL", "Redis"],
  }),
  service(northbeanProduction, "order-worker", "github", { dependsOn: ["PostgreSQL", "Redis"] }),
  service(northbeanProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(northbeanProduction, "Redis", "redis", { volume: "redis-data" }),
  service(northbeanStaging, "shop-api", "github", {
    status: "deploying",
    domains: ["api.staging.northbean.coffee"],
    dependsOn: ["PostgreSQL"],
  }),
  service(northbeanStaging, "PostgreSQL", "postgres", { volume: "postgres-data" }),

  service(routelaneProduction, "dashboard", "static", { domains: ["routelane.app"] }),
  service(routelaneProduction, "tracking-api", "github", {
    domains: ["api.routelane.app"],
    dependsOn: ["PostgreSQL", "MongoDB", "Redis"],
  }),
  service(routelaneProduction, "gps-ingest", "github", { dependsOn: ["MongoDB", "Redis"] }),
  service(routelaneProduction, "sms-gateway", "image", {
    status: "crashed",
    dependsOn: ["Redis"],
  }),
  service(routelaneProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(routelaneProduction, "MongoDB", "mongodb", { volume: "mongodb-data" }),
  service(routelaneProduction, "Redis", "redis", { volume: "redis-data" }),

  service(deskmindProduction, "widget", "static", { domains: ["deskmind.app"] }),
  service(deskmindProduction, "support-api", "github", {
    domains: ["api.deskmind.app"],
    dependsOn: ["PostgreSQL", "Redis", "inference"],
  }),
  service(deskmindProduction, "inference", "image", {
    volume: "model-weights",
    mountPath: "/root/.cache/huggingface",
  }),
  service(deskmindProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(deskmindProduction, "Redis", "redis", { volume: "redis-data" }),

  service(tallybookProduction, "web-app", "static", { domains: ["app.tallybook.cloud"] }),
  service(tallybookProduction, "billing-api", "github", {
    domains: ["api.tallybook.cloud"],
    dependsOn: ["PostgreSQL", "Redis", "pdf-renderer"],
  }),
  service(tallybookProduction, "pdf-renderer", "image"),
  service(tallybookProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(tallybookProduction, "Redis", "redis", { volume: "redis-data" }),
  service(tallybookStaging, "billing-api", "github", {
    domains: ["api.staging.tallybook.cloud"],
    dependsOn: ["PostgreSQL"],
  }),
  service(tallybookStaging, "PostgreSQL", "postgres", { volume: "postgres-data" }),

  service(clinicslotProduction, "booking-web", "static", { domains: ["clinicslot.app"] }),
  service(clinicslotProduction, "booking-api", "github", {
    domains: ["api.clinicslot.app"],
    dependsOn: ["PostgreSQL", "Redis"],
  }),
  service(clinicslotProduction, "reminders-worker", "github", {
    dependsOn: ["PostgreSQL", "Redis"],
  }),
  service(clinicslotProduction, "PostgreSQL", "postgres", { volume: "postgres-data" }),
  service(clinicslotProduction, "Redis", "redis", { volume: "redis-data" }),
];

export type Runtime = "node" | "go" | "python";

// How the server runs an app built from a repository.
export const runtimes: Readonly<
  Record<Runtime, { readonly startCommand: string; readonly port: number }>
> = {
  node: { startCommand: "pnpm start", port: 3000 },
  go: { startCommand: "./server", port: 8080 },
  python: { startCommand: "uvicorn app.main:app --host 0.0.0.0 --port 8000", port: 8000 },
};

interface AppCommit {
  readonly sha: string;
  readonly message: string;
  // What the build printed when it broke: this commit never went live.
  readonly failed?: string;
}

// What the server knows of an app beyond the service: where it comes from and what it went
// through. Keyed by service name, shared by the environments of a project.
interface App {
  readonly repository?: string;
  readonly runtime?: Runtime;
  // Newest first.
  readonly commits?: readonly AppCommit[];
  readonly images?: readonly string[];
}

export const apps: Readonly<Record<string, App>> = {
  "shop-api": {
    repository: "northbean/shop",
    runtime: "node",
    commits: [
      { sha: "d63eac96f1b2", message: "feat(checkout): support Apple Pay and Google Pay" },
      { sha: "a41c0e7d9b30", message: "perf(catalog): cache the product list for 60 seconds" },
      { sha: "7f2b9c1e5d44", message: "fix(cart): keep the cart when a guest signs in" },
      {
        sha: "c90d4a2f1e87",
        message: "feat(subscriptions): pause and resume a subscription",
        failed:
          "src/subscriptions/pause.ts(31,18): error TS2339: Property 'resumeAt' does not exist on type 'Subscription'.",
      },
      { sha: "3e8f6b0a7c12", message: "chore: bump dependencies" },
    ],
  },
  "order-worker": {
    repository: "northbean/shop",
    runtime: "node",
    commits: [
      { sha: "5b1d7e0c4a96", message: "feat(emails): send the tracking link when an order ships" },
      { sha: "e2a9f4c81b37", message: "fix(stock): release the stock held by abandoned carts" },
      { sha: "08c6b3d5f2e1", message: "fix(payments): retry the Stripe webhooks with backoff" },
    ],
  },
  storefront: {
    repository: "northbean/storefront",
    runtime: "node",
    commits: [
      { sha: "9a4e2c7b0d13", message: "feat(product): show the roast date and the tasting notes" },
      { sha: "1f7c5e9a3b28", message: "fix(seo): canonical URLs on the product pages" },
      { sha: "b6d0a8e4c751", message: "chore: bump vite" },
    ],
  },
  "tracking-api": {
    repository: "routelane/tracking",
    runtime: "node",
    commits: [
      { sha: "4c8e1a6f2d90", message: "feat(trips): estimated arrival time on every stop" },
      { sha: "f3b7d2e95c04", message: "fix(alerts): one alert per geofence crossing, not two" },
      { sha: "6a0d9c3e7b15", message: "perf(positions): index the positions by vehicle and time" },
      {
        sha: "2e5f8b1c4a63",
        message: "feat(reports): monthly fuel report",
        failed: "src/reports/fuel.ts(18,9): error TS2554: Expected 2 arguments, but got 1.",
      },
      { sha: "8d1a4f7c0e29", message: "chore: bump dependencies" },
    ],
  },
  "gps-ingest": {
    repository: "routelane/tracking",
    runtime: "go",
    commits: [
      { sha: "c7e3a0b5d824", message: "perf(ingest): insert the positions in batches of 1000" },
      { sha: "0b9f6d2a8e41", message: "fix(ingest): drop the positions older than 24 hours" },
      { sha: "d5a2c8e0f316", message: "feat(ingest): accept Teltonika codec 8 extended" },
    ],
  },
  "sms-gateway": {
    images: ["ghcr.io/routelane/sms-gateway:1.4.2", "ghcr.io/routelane/sms-gateway:1.4.1"],
  },
  dashboard: {
    repository: "routelane/dashboard",
    runtime: "node",
    commits: [
      { sha: "7b2e9d4a1c06", message: "feat(map): cluster the vehicles at low zoom" },
      {
        sha: "e8c1f5b3a927",
        message: "fix(map): keep the selected vehicle when the list refreshes",
      },
      { sha: "3a6d0e8c2f54", message: "chore: bump vite" },
    ],
  },
  "support-api": {
    repository: "deskmind/support",
    runtime: "python",
    commits: [
      {
        sha: "a9d3f1c7e520",
        message: "feat(answers): cite the documentation pages in each answer",
      },
      {
        sha: "5e0b8a2d6c14",
        message: "fix(tickets): hand over to a human after two failed answers",
      },
      { sha: "c2f7e4a9b083", message: "feat(rag): index the help center in pgvector" },
      {
        sha: "1d8b5c0e3a72",
        message: "feat(voice): answer phone calls",
        failed: "× Failed to build `pyaudio==0.2.14`: portaudio.h: No such file or directory",
      },
      { sha: "9e4a2f6d1b38", message: "chore: bump fastapi" },
    ],
  },
  inference: { images: ["vllm/vllm-openai:v0.11.0", "vllm/vllm-openai:v0.10.2"] },
  widget: {
    repository: "deskmind/widget",
    runtime: "node",
    commits: [
      { sha: "f0c6a3e8d215", message: "feat(widget): stream the answers as they are written" },
      { sha: "4b9e1d7a2c60", message: "fix(widget): load after the host page, not before" },
      { sha: "8e2c5f0b9d47", message: "chore: bump dependencies" },
    ],
  },
  "billing-api": {
    repository: "tallybook/billing",
    runtime: "go",
    commits: [
      { sha: "b3e8d0a5f192", message: "feat(invoices): e-invoicing export in Factur-X" },
      { sha: "6f1a9c4e2d07", message: "fix(reminders): skip the invoices paid by bank transfer" },
      { sha: "d0c7b2e6a834", message: "perf(pdf): render the invoices in parallel" },
      {
        sha: "2a5e8f1d9c63",
        message: "feat(quotes): turn a quote into an invoice",
        failed:
          "./internal/quotes/convert.go:57:14: cannot use q.Total (variable of type float64) as Money value in assignment",
      },
      { sha: "e9b4c0a7f521", message: "chore: bump go to 1.25" },
    ],
  },
  "pdf-renderer": { images: ["gotenberg/gotenberg:8.21.1", "gotenberg/gotenberg:8.20.0"] },
  "web-app": {
    repository: "tallybook/app",
    runtime: "node",
    commits: [
      { sha: "7c0e5a2f8b16", message: "feat(dashboard): the overdue invoices at a glance" },
      { sha: "a3f9d6b1e042", message: "fix(invoice-form): VAT rounding on multi-line invoices" },
      { sha: "5d2b8e0c7a39", message: "chore: bump vite" },
    ],
  },
  "booking-api": {
    repository: "clinicslot/booking",
    runtime: "python",
    commits: [
      { sha: "e6a1c9f3d058", message: "feat(slots): book a follow-up from the appointment page" },
      { sha: "0f8d4b2e6c91", message: "fix(timezones): appointments across the DST change" },
      { sha: "b9c3e7a0d524", message: "feat(calendar): sync with Google and Outlook calendars" },
      {
        sha: "3d7f0a5c9e12",
        message: "feat(video): teleconsultation links",
        failed: "× Failed to build `av==14.0.1`: ffmpeg development headers not found",
      },
      { sha: "c1e8b6d2f740", message: "chore: bump django to 5.2" },
    ],
  },
  "reminders-worker": {
    repository: "clinicslot/booking",
    runtime: "python",
    commits: [
      { sha: "8a5d1f9c3e06", message: "feat(reminders): SMS 24 hours and 2 hours before" },
      { sha: "2c9e6b0a4d71", message: "fix(reminders): no reminder for a cancelled appointment" },
      { sha: "f4b0d8e3a259", message: "perf(reminders): send in batches of 100" },
    ],
  },
  "booking-web": {
    repository: "clinicslot/web",
    runtime: "node",
    commits: [
      { sha: "6e3a0c8f1d95", message: "feat(booking): pick a practitioner, then a slot" },
      { sha: "d1f7b4e9a026", message: "fix(a11y): keyboard navigation in the calendar" },
      { sha: "9b2c6e0f5a83", message: "chore: bump vite" },
    ],
  },
};

const appRepositories = Object.values(apps).flatMap(({ repository }) =>
  repository ? [repository] : [],
);

// What the GitHub app can read: the deployed repositories, and a few that are not yet.
export const repositories: readonly Repository[] = [
  ...new Set([
    ...appRepositories,
    "northbean/wholesale-portal",
    "routelane/driver-app",
    "deskmind/evals",
  ]),
].map((fullName) => ({ fullName }));

// Every repository has the branches of the seeded environments; northbean/shop has a few more.
export function branchesOf(repository: string): string[] {
  const common = ["main", "dev", "staging"];
  return repository === "northbean/shop"
    ? [...common, "develop", "feat/subscriptions", "fix/cart-merge"]
    : common;
}
