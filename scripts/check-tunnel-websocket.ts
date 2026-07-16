const url = Bun.argv[2];
if (!url) {
  throw new Error("usage: bun check-tunnel-websocket.ts <ws-url>");
}

await new Promise<void>((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error("WebSocket check timed out")), 5000);
  const socket = new WebSocket(url, "brume-echo");
  socket.addEventListener("open", () => {
    if (socket.protocol !== "brume-echo") {
      reject(new Error(`unexpected WebSocket protocol: ${socket.protocol}`));
      return;
    }
    socket.send("brume-websocket-e2e");
  });
  socket.addEventListener("message", (event) => {
    if (event.data !== "brume-websocket-e2e") {
      reject(new Error(`unexpected WebSocket message: ${event.data}`));
      return;
    }
    clearTimeout(timeout);
    socket.close();
    resolve();
  });
  socket.addEventListener("error", () => reject(new Error("WebSocket check failed")));
});

console.log("Tunnel WebSocket check passed");
