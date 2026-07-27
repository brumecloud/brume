import { defineConfig } from "vite";
import { readFileSync } from "node:fs";

export default defineConfig({
  plugins: [
    {
      name: "emit-geist-font",
      generateBundle() {
        this.emitFile({
          type: "asset",
          fileName: "geist.woff2",
          source: readFileSync(
            "node_modules/@fontsource-variable/geist/files/geist-latin-wght-normal.woff2",
          ),
        });
      },
    },
  ],
  build: {
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
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
