import type { Plugin, UserConfig } from "vite-plus";

// `vp dev` talks to the API of `brume dev` on the same origin, through this proxy. `vp dev --mode
// mock` keeps the mocked API instead, and the tests and the brume.dev demo never leave MSW. The
// API's Host allowlist has port 5173 for the dev server, no other: when the port is taken, Vite
// fails to start rather than move to one the API would refuse. It listens on 127.0.0.1, the host
// of the login link `brume dev` prints: the session cookie knows hosts, not ports, so it carries
// over from 7878 to 5173.
export function realApiServer(mode: string): UserConfig {
  return mode === "development"
    ? {
        server: {
          host: "127.0.0.1",
          port: 5173,
          strictPort: true,
          proxy: { "/api": "http://127.0.0.1:7878" },
        },
      }
    : {};
}

export function realApi(): Plugin {
  return {
    name: "brume:real-api",
    apply: "serve",
    config: (_config, { mode }) => realApiServer(mode),
  };
}
