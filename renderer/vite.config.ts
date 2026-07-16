import { defineConfig } from "vite";

export default defineConfig({
  build: {
    emptyOutDir: false,
    lib: {
      entry: "src/client/runtime.ts",
      formats: ["es"],
      fileName: () => "runtime.js",
    },
    outDir: "dist/web",
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
        assetFileNames: (asset) =>
          asset.names.some((name) => name.endsWith(".css"))
            ? "theme.css"
            : "assets/[name]-[hash][extname]",
      },
    },
  },
});
