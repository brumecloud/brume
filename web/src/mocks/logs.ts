import type {
  Deployment,
  DeploymentLogStream,
  LogLevel,
  LogLine,
  Service,
  ServiceKind,
} from "~/api/types";

// What the services of the seed print. Some lines are hostile on purpose: the panel must
// show them as text, colors kept and every other escape sequence dropped.
const ESC = "\u001B";
const green = (text: string) => `${ESC}[32m${text}${ESC}[0m`;
const yellow = (text: string) => `${ESC}[33m${text}${ESC}[0m`;
const red = (text: string) => `${ESC}[1;31m${text}${ESC}[0m`;
const gray = (text: string) => `${ESC}[90m${text}${ESC}[0m`;
const cyan = (text: string) => `${ESC}[36m${text}${ESC}[0m`;

type Template = readonly [LogLevel, string];

const TEMPLATES: Record<ServiceKind, readonly Template[]> = {
  github: [
    ["info", `${green("INFO")} GET /api/projects 200 ${gray("12ms")}`],
    ["info", `${green("INFO")} POST /api/runs 201 ${gray("48ms")}`],
    ["info", `${green("INFO")} GET /health 200 ${gray("1ms")}`],
    ["warn", `${yellow("WARN")} slow query on executions ${gray("(412ms)")}`],
    ["error", `${red("ERROR")} Redis connection reset, retrying in 1s`],
    ["info", `${green("INFO")} GET /search?q=<script>alert(1)</script> 404 ${gray("3ms")}`],
  ],
  static: [
    ["info", '10.0.0.4 - - "GET / HTTP/1.1" 200 5123 "-" "Mozilla/5.0"'],
    ["info", '10.0.0.4 - - "GET /assets/index-DgjfHeup.js HTTP/1.1" 200 140880'],
    ["warn", '10.0.0.9 - - "GET /.env HTTP/1.1" 404 153 "-" "curl/8.4.0"'],
  ],
  image: [
    ["error", `${red("panic")}: dial tcp redis:6379: connect: connection refused`],
    ["info", "wgengine: Reconfig: configuring userspace WireGuard config (with 1/4 peers)"],
    ["info", `${ESC}[2K${ESC}[1Gmagicsock: derp-12 connected${ESC}]0;pwned\u0007`],
  ],
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

let sequence = 0;
// Each service goes through its own templates in turn.
const printed = new Map<string, number>();

function lineFor(service: Service, timestamp: number): LogLine {
  sequence += 1;
  const count = printed.get(service.id) ?? 0;
  printed.set(service.id, count + 1);
  const templates = TEMPLATES[service.kind];
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

const BUILD = [
  `${ESC}[1mRailpack 0.40.1${ESC}[0m`,
  `${cyan("↳")} Detected Node 22 with pnpm 10`,
  `${cyan("↳")} Using build command ${ESC}[1mpnpm build${ESC}[0m`,
  "$ pnpm install --frozen-lockfile",
  "Packages: +812",
  "Progress: resolved 812, reused 790, downloaded 22, added 812, done",
  "$ pnpm build",
  "vite v8.0.0 building for production...",
  "✓ 2531 modules transformed.",
];

export function deploymentLogs(
  deployment: Deployment,
  service: Service,
  stream: DeploymentLogStream,
): LogLine[] {
  const failed = deployment.status === "failed";
  const messages: readonly (readonly [LogLevel, string])[] =
    stream === "build"
      ? deployment.commit
        ? [
            ...BUILD.map((message) => ["info", message] as const),
            failed
              ? (["error", `${red("✗")} Build failed: src/index.ts(12,3): error TS2322`] as const)
              : (["info", `${green("✓")} Image built in 41s`] as const),
          ]
        : [["info", `Pas de build : image ${deployment.image ?? ""} tirée telle quelle.`]]
      : failed
        ? [["info", "Aucun container lancé : le build a échoué."]]
        : [
            ["info", `Starting container ${service.slug}`],
            ["info", `${green("INFO")} Listening on :3000`],
            ["info", `Healthcheck /health → 200 in ${gray("14ms")}`],
            ["info", `${green("✓")} Healthy, traffic switched to this deployment`],
          ];
  const start = Date.parse(deployment.createdAt);
  return messages.map(([level, message], index) => ({
    id: `${deployment.id}-${stream}-${index}`,
    timestamp: new Date(start + index * 2000).toISOString(),
    serviceId: deployment.serviceId,
    level,
    message,
  }));
}
