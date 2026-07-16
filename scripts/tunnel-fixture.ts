const port = Number(Bun.env.TUNNEL_FIXTURE_PORT ?? "18081");
const instance = Bun.env.TUNNEL_FIXTURE_NAME ?? "fixture";

const server = Bun.serve({
  hostname: "127.0.0.1",
  port,
  async fetch(request, server) {
    const url = new URL(request.url);
    if (url.pathname === "/ws") {
      const requestedProtocol = request.headers
        .get("sec-websocket-protocol")
        ?.split(",")[0]
        ?.trim();
      const upgraded = server.upgrade(request, {
        headers: requestedProtocol
          ? { "Sec-WebSocket-Protocol": requestedProtocol }
          : undefined,
      });
      if (upgraded) {
        return undefined;
      }
      return new Response("WebSocket upgrade failed", { status: 400 });
    }
    if (url.pathname === "/stream") {
      return new Response(
        new ReadableStream({
          async start(controller) {
            controller.enqueue("alpha-");
            await Bun.sleep(25);
            controller.enqueue("beta");
            controller.close();
          },
        }),
        { headers: { "x-local-response": "stream" } },
      );
    }
    if (url.pathname === "/redirect") {
      return new Response(null, {
        status: 302,
        headers: {
          location: "/landing",
          "set-cookie": "session=fixture; Path=/; HttpOnly",
        },
      });
    }

    const body = await request.text();
    return Response.json(
      {
        instance,
        method: request.method,
        pathname: url.pathname,
        search: url.search,
        body,
        host: request.headers.get("host"),
        forwardedHost: request.headers.get("x-forwarded-host"),
        forwardedProto: request.headers.get("x-forwarded-proto"),
        forwardedPrefix: request.headers.get("x-forwarded-prefix"),
        testHeader: request.headers.get("x-tunnel-test"),
      },
      {
        status: url.pathname === "/submit" ? 202 : 200,
        headers: { "x-local-response": instance },
      },
    );
  },
  websocket: {
    message(socket, message) {
      socket.send(message);
    },
  },
});

console.log(`Tunnel fixture ${instance} listening on ${server.url}`);
