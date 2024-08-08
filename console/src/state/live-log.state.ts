import { proxy } from "valtio";

export const liveLogs = proxy({
  isLive: false,
  startLive: () => (liveLogs.isLive = true),
  stopLive: () => (liveLogs.isLive = false),
  toggleLive: () => (liveLogs.isLive = !liveLogs.isLive),
});
