import { proxy } from "valtio";

export const liveLogs = proxy({
  isLive: false,
  searchQuery: "",
  updateSearchQuery: (text: string) => (liveLogs.searchQuery = text),
  startLive: () => (liveLogs.isLive = true),
  stopLive: () => (liveLogs.isLive = false),
  toggleLive: () => (liveLogs.isLive = !liveLogs.isLive),
});
