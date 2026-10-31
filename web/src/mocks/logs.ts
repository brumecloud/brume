import type {
  Deployment,
  DeploymentLogStream,
  LogLevel,
  LogLine,
  Service,
  ServiceKind,
} from "~/api/types";

import { type Runtime, apps, runtimes } from "./seed";

// What the services of the seed print. Some lines are hostile on purpose: the panel must
// show them as text, colors kept and every other escape sequence dropped.
const ESC = "\u001B";
const green = (text: string) => `${ESC}[32m${text}${ESC}[0m`;
const yellow = (text: string) => `${ESC}[33m${text}${ESC}[0m`;
const red = (text: string) => `${ESC}[1;31m${text}${ESC}[0m`;
const gray = (text: string) => `${ESC}[90m${text}${ESC}[0m`;
const cyan = (text: string) => `${ESC}[36m${text}${ESC}[0m`;

type Template = readonly [LogLevel, string];

const request = (method: string, path: string, status: number, time: string, extra = "") =>
  `${green("INFO")} ${method} ${path} ${status} ${gray(time)}${extra ? ` ${extra}` : ""}`;

const staticSite = (asset: string): readonly Template[] => [
  ["info", '10.0.0.4 - - "GET / HTTP/1.1" 200 5123 "-" "Mozilla/5.0"'],
  ["info", `10.0.0.4 - - "GET /assets/${asset} HTTP/1.1" 200 140880`],
  ["warn", '10.0.0.9 - - "GET /.env HTTP/1.1" 404 153 "-" "curl/8.4.0"'],
];

// What a service prints when its app has no lines of its own: a database, or a new service.
const TEMPLATES: Record<ServiceKind, readonly Template[]> = {
  github: [
    ["info", request("GET", "/api/items", 200, "12ms")],
    ["info", request("POST", "/api/items", 201, "48ms")],
    ["info", request("GET", "/health", 200, "1ms")],
  ],
  static: staticSite("index-DgjfHeup.js"),
  image: [["info", "Container started"]],
  postgres: [
    ["info", "LOG:  checkpoint starting: time"],
    ["info", "LOG:  checkpoint complete: wrote 12 buffers (0.1%); 0 WAL file(s) added"],
    ["warn", "WARNING:  there is no transaction in progress"],
  ],
  mongodb: [
    [
      "info",
      '{"t":{"$date":"2026-10-28T22:48:25.829+00:00"},"s":"I","c":"WTCHKPT","msg":"WiredTiger message","attr":{"message":{"ts_sec":1793227705,"category":"WT_VERB_CHECKPOINT_PROGRESS","msg":"saving checkpoint snapshot min: 30572"}}}',
    ],
    [
      "info",
      '{"s":"I","c":"NETWORK","msg":"Connection accepted","attr":{"remote":"10.0.1.7:51834"}}',
    ],
  ],
  redis: [
    ["info", "1:M 28 Oct 2026 22:48:04.014 * 1 changes in 60 seconds. Saving..."],
    ["info", "1:M 28 Oct 2026 22:48:04.014 * Background saving started by pid 24218"],
    ["info", "24218:C 28 Oct 2026 22:48:04.023 * DB saved on disk"],
    ["info", "1:M 28 Oct 2026 22:48:04.115 * Background saving terminated with success"],
  ],
};

// What each app of the seed prints, by service name.
const APP_LINES: Readonly<Record<string, readonly Template[]>> = {
  "shop-api": [
    ["info", request("POST", "/v1/cart/items", 201, "18ms")],
    ["info", request("GET", "/v1/products?collection=espresso", 200, "24ms")],
    ["info", request("GET", "/health", 200, "1ms")],
    ["info", request("POST", "/v1/checkout", 201, "212ms")],
    ["warn", `${yellow("WARN")} low stock: ethiopia-yirgacheffe-250g (4 left)`],
    ["info", request("GET", "/v1/search?q=<script>alert(1)</script>", 200, "9ms")],
    ["warn", `${yellow("WARN")} slow query on orders ${gray("(412ms)")}`],
    ["error", `${red("ERROR")} Redis connection reset, retrying in 1s`],
  ],
  "order-worker": [
    ["info", `${green("INFO")} order 10482 paid: stock reserved, confirmation email sent`],
    ["info", `${green("INFO")} order 10479 shipped: tracking link sent`],
    ["warn", `${yellow("WARN")} Stripe webhook for order 10481 received twice, ignored`],
    ["info", `${green("INFO")} queue empty, waiting`],
  ],
  storefront: staticSite("index-Bq3gA7xe.js"),
  "tracking-api": [
    ["info", request("GET", "/v1/vehicles?status=moving", 200, "33ms")],
    ["info", request("GET", "/v1/trips/T-8821/eta", 200, "6ms")],
    ["info", request("GET", "/health", 200, "1ms")],
    ["warn", `${yellow("WARN")} geofence: vehicle AB-123-CD left the Rungis depot after hours`],
    ["info", request("POST", "/v1/alerts/ack", 204, "4ms")],
  ],
  "gps-ingest": [
    ["info", `ingest: 1000 positions written in ${gray("41ms")} (batch 88213)`],
    ["info", "ingest: 214 vehicles reporting, 3 silent for more than 10 min"],
    ["warn", "ingest: position from vehicle 0417 is dated in the future, dropped"],
  ],
  "sms-gateway": [
    ["info", "sms-gateway 1.4.2 starting with 2 workers"],
    ["info", `${ESC}[2K${ESC}[1Gprovider: connecting to the SMS API${ESC}]0;pwned\u0007`],
    ["error", `${red("fatal")}: the SMS provider rejected the API key (401 Unauthorized)`],
  ],
  dashboard: staticSite("index-C4rWb9sQ.js"),
  "support-api": [
    [
      "info",
      '{"level":"info","event":"answer.sent","ticket":"T-20931","sources":3,"latency_ms":2410}',
    ],
    ["info", '{"level":"info","event":"rag.retrieved","passages":4,"latency_ms":12}'],
    [
      "warn",
      '{"level":"warning","event":"ticket.handover","ticket":"T-20934","reason":"low confidence"}',
    ],
    ["info", request("GET", "/health", 200, "1ms")],
  ],
  inference: [
    [
      "info",
      "INFO: Avg prompt throughput: 812.4 tokens/s, Avg generation throughput: 96.1 tokens/s, Running: 3 reqs",
    ],
    ["info", "INFO: GPU KV cache usage: 41.3%, prefix cache hit rate: 63.0%"],
    ["info", 'INFO: 10.0.4.2:41870 - "POST /v1/chat/completions HTTP/1.1" 200 OK'],
  ],
  widget: staticSite("index-Pt8dK1vz.js"),
  "billing-api": [
    ["info", request("POST", "/v1/invoices", 201, "46ms")],
    ["info", request("GET", "/v1/invoices?status=overdue", 200, "22ms")],
    ["info", `${green("INFO")} invoice INV-2026-0412 rendered to PDF in ${gray("380ms")}`],
    ["warn", `${yellow("WARN")} INV-2026-0388 is 30 days overdue: second reminder sent`],
  ],
  "pdf-renderer": [
    [
      "info",
      '{"level":"info","logger":"api","message":"request handled","path":"/forms/chromium/convert/html","status":200,"latency_human":"381ms"}',
    ],
    ["info", '{"level":"info","logger":"chromium","message":"browser restarted"}'],
  ],
  "web-app": staticSite("index-Fx2nL5pe.js"),
  "booking-api": [
    ["info", 'INFO "POST /api/appointments HTTP/1.1" 201 (54ms)'],
    ["info", 'INFO "GET /api/practitioners/42/slots?date=2026-11-03 HTTP/1.1" 200 (17ms)'],
    ["warn", "WARNING slot 2026-11-03T09:30 booked twice at once: second request rejected"],
    ["info", 'INFO "GET /health HTTP/1.1" 200 (1ms)'],
  ],
  "reminders-worker": [
    ["info", "[INFO] Task reminders.send_sms[8f2c41] succeeded in 0.21s"],
    ["info", "[INFO] 312 reminders scheduled for tomorrow"],
    ["warn", "[WARNING] +33 6 •• •• •• 17 unreachable, reminder sent by email instead"],
  ],
  "booking-web": staticSite("index-Fl4nA0rn.js"),
};

let sequence = 0;
// Each service goes through its own templates in turn.
const printed = new Map<string, number>();

function lineFor(service: Service, timestamp: number): LogLine {
  sequence += 1;
  const count = printed.get(service.id) ?? 0;
  printed.set(service.id, count + 1);
  const templates = APP_LINES[service.name] ?? TEMPLATES[service.kind];
  const [level, message] = templates[count % templates.length] ?? ["info", ""];
  return {
    id: `log-${sequence}`,
    timestamp: new Date(timestamp).toISOString(),
    serviceId: service.id,
    level,
    message,
  };
}

// `count` lines spread over the last `spanMs`, oldest first.
export function backlog(services: readonly Service[], count: number, spanMs: number): LogLine[] {
  const now = Date.now();
  return Array.from({ length: services.length === 0 ? 0 : count }, (_, index) => {
    const service = services[index % services.length];
    return service ? lineFor(service, now - spanMs + (index * spanMs) / count) : undefined;
  }).filter((line) => line !== undefined);
}

export function liveLine(services: readonly Service[]): LogLine | undefined {
  const service = services[sequence % Math.max(services.length, 1)];
  return service ? lineFor(service, Date.now()) : undefined;
}

const step = (text: string) => `${cyan("↳")} ${text}`;
const bold = (text: string) => `${ESC}[1m${text}${ESC}[0m`;
const RAILPACK = bold("Railpack 0.40.1");

// What the build of each kind of app prints.
const BUILDS: Record<Runtime | "static", readonly string[]> = {
  node: [
    RAILPACK,
    step("Detected Node 22 with pnpm 10"),
    step(`Using build command ${bold("pnpm build")}`),
    "$ pnpm install --frozen-lockfile",
    "Packages: +412",
    "Progress: resolved 412, reused 398, downloaded 14, added 412, done",
    "$ pnpm build",
    "> tsc -p tsconfig.build.json",
  ],
  go: [
    RAILPACK,
    step("Detected Go 1.25"),
    step(`Using build command ${bold("go build -o server ./cmd/server")}`),
    "$ go mod download",
    "$ go build -o server ./cmd/server",
  ],
  python: [
    RAILPACK,
    step("Detected Python 3.13 with uv"),
    "$ uv sync --frozen --no-dev",
    "Resolved 64 packages in 3ms",
    "Prepared 12 packages in 1.82s",
    "Installed 64 packages in 1.21s",
  ],
  static: [
    RAILPACK,
    step("Detected a static site built with Vite"),
    step(`Using build command ${bold("pnpm build")}`),
    "$ pnpm install --frozen-lockfile",
    "Packages: +812",
    "Progress: resolved 812, reused 790, downloaded 22, added 812, done",
    "$ pnpm build",
    "vite v8.0.0 building for production...",
    "✓ 2531 modules transformed.",
  ],
};

function buildLines(deployment: Deployment, service: Service): readonly Template[] {
  if (!deployment.commit) {
    return [["info", `No build: image ${deployment.image ?? ""} pulled as is.`]];
  }
  const { commit } = deployment;
  const app = apps[service.name];
  const error = app?.commits?.find(({ sha }) => sha === commit.sha)?.failed;
  return [
    ...BUILDS[service.kind === "static" ? "static" : (app?.runtime ?? "node")].map(
      (line) => ["info", line] as const,
    ),
    deployment.status === "failed"
      ? ["error", `${red("✗")} Build failed: ${error ?? "see the lines above"}`]
      : ["info", `${green("✓")} Image built in 41s`],
  ];
}

function deployLines(deployment: Deployment, service: Service): readonly Template[] {
  if (deployment.status === "failed") {
    return [["info", "No container started: the build failed."]];
  }
  const healthy: Template = ["info", `${green("✓")} Healthy, traffic switched to this deployment`];
  if (service.kind === "static") {
    return [
      ["info", `Starting container ${service.slug}`],
      ["info", `Caddy serving /app/dist on ${gray(":80")}`],
      healthy,
    ];
  }
  const runtime = apps[service.name]?.runtime;
  if (service.kind !== "github" || !runtime) {
    return [["info", `Starting container ${service.slug}`], healthy];
  }
  return [
    ["info", `Starting container ${service.slug}`],
    ["info", `${green("INFO")} Listening on :${runtimes[runtime].port}`],
    ["info", `Healthcheck /health → 200 in ${gray("14ms")}`],
    healthy,
  ];
}

export function deploymentLogs(
  deployment: Deployment,
  service: Service,
  stream: DeploymentLogStream,
): LogLine[] {
  const messages =
    stream === "build" ? buildLines(deployment, service) : deployLines(deployment, service);
  const start = Date.parse(deployment.createdAt);
  return messages.map(([level, message], index) => ({
    id: `${deployment.id}-${stream}-${index}`,
    timestamp: new Date(start + index * 2000).toISOString(),
    serviceId: deployment.serviceId,
    level,
    message,
  }));
}
