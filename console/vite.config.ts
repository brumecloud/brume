import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
      "@shadcn": path.resolve(__dirname, "src/components/ui"),
      src: path.resolve(__dirname, "src"),
    },
  },
});
