import { once } from "node:events";
import { PassThrough } from "node:stream";

import { describe, expect, it } from "vite-plus/test";

import { endWithUpstream, realApiServer } from "./real-api.ts";

describe("realApiServer", () => {
  it("serves on 127.0.0.1:5173 and proxies /api to brume dev", () => {
    // The login link of `brume dev` opens a session on 127.0.0.1: a cookie knows hosts, not
    // ports, so the dev server must be on the same host for the session to carry over.
    expect(realApiServer("development")).toEqual({
      server: {
        host: "127.0.0.1",
        port: 5173,
        strictPort: true,
        proxy: { "/api": { target: "http://127.0.0.1:7878", configure: expect.any(Function) } },
      },
    });
  });

  it("leaves the mock and demo modes alone", () => {
    expect(realApiServer("mock")).toEqual({});
    expect(realApiServer("demo")).toEqual({});
  });
});

// The browser's side of the response.
const downstream = () => ({
  writableEnded: false,
  destroyed: false,
  destroy() {
    this.destroyed = true;
  },
});

describe("endWithUpstream", () => {
  it("drops the browser's response when brume dev goes away mid-stream", async () => {
    const upstream = new PassThrough();
    const response = downstream();
    endWithUpstream(upstream, response);

    upstream.destroy();
    await once(upstream, "close");

    expect(response.destroyed).toBe(true);
  });

  it("leaves a response that ended normally alone", async () => {
    const upstream = new PassThrough();
    const response = downstream();
    endWithUpstream(upstream, response);
    upstream.resume();

    const closed = once(upstream, "close");
    upstream.end("done");
    await closed;

    expect(response.destroyed).toBe(false);
  });
});
