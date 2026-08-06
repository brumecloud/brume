import { defineConfig } from "vite";

export default defineConfig({
  build: {
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    emptyOutDir: false,
    lib: {
      entry: "src/overlay/overlay.ts",
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
