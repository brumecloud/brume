import path from "node:path";
import react from "@vitejs/plugin-react";
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
  server: {
    watch: {
      usePolling: true,
      interval: 500,
    },
    host: "0.0.0.0",
    strictPort: true,
  },
  optimizeDeps: {
    exclude: ["@graphql-typed-document-node/core"],
    include: ["@_apollo/"],
  },
});
