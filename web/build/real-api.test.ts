import { describe, expect, it } from "vite-plus/test";

import { realApiServer } from "./real-api.ts";

describe("realApiServer", () => {
  it("serves on 127.0.0.1:5173 and proxies /api to brume dev", () => {
    // The login link of `brume dev` opens a session on 127.0.0.1: a cookie knows hosts, not
    // ports, so the dev server must be on the same host for the session to carry over.
    expect(realApiServer("development")).toEqual({
      server: {
        host: "127.0.0.1",
        port: 5173,
        strictPort: true,
        proxy: { "/api": "http://127.0.0.1:7878" },
      },
    });
  });

  it("leaves the mock and demo modes alone", () => {
    expect(realApiServer("mock")).toEqual({});
    expect(realApiServer("demo")).toEqual({});
  });
});
