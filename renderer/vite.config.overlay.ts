import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss()],
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  define: {
    // Vite lib mode does not substitute NODE_ENV; without this the IIFE ships
    // react-dom dev mode and crashes on `process` in the browser.
    "process.env.NODE_ENV": JSON.stringify("production"),
  },
  build: {
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    emptyOutDir: false,
    lib: {
      entry: "src/overlay/overlay.tsx",
      formats: ["iife"],
      name: "BrumeOverlay",
      fileName: () => "overlay.js",
    },
    outDir: "dist/web",
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
  },
});
