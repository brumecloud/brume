import { fileURLToPath } from "node:url";

import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { msw } from "msw/vite";
import { defineConfig } from "vite-plus";
import { playwright } from "vite-plus/test/browser-playwright";

import { noChunkCycles } from "./build/no-chunk-cycles.ts";

interface ImportPattern {
  readonly group: string[];
  readonly importNames?: string[];
  readonly allowTypeImports?: boolean;
  readonly message: string;
}

// Class recipes stay in components/ui: render the component instead.
const uiRecipes: ImportPattern = {
  group: ["~/components/ui/*"],
  importNames: ["buttonVariants", "badgeVariants", "toggleVariants"],
  message: "Render the component from components/ui instead of borrowing its recipe.",
};

// Imports flow down the layers: routes → components → helpers.
const layerRules = (...patterns: ImportPattern[]): ["error", { patterns: ImportPattern[] }] => [
  "error",
  { patterns: [uiRecipes, ...patterns] },
];

export default defineConfig({
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: true }),
    react(),
    babel({ presets: [reactCompilerPreset({ panicThreshold: "none" })] }),
    tailwindcss(),
    msw(),
    noChunkCycles(),
  ],
  resolve: {
    alias: { "~": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  fmt: {
    ignorePatterns: ["dist/**", "src/routeTree.gen.ts", "pnpm-lock.yaml"],
  },
  lint: {
    ignorePatterns: ["dist/**", "src/routeTree.gen.ts"],
    plugins: ["eslint", "oxc", "react", "unicorn", "typescript"],
    jsPlugins: ["./oxlint-plugin/index.ts", "@shadcn/lint"],
    settings: { shadcn: { ui: "~/components/ui" } },
    categories: { correctness: "error", suspicious: "warn", perf: "warn" },
    options: { reportUnusedDisableDirectives: "error" },
    rules: {
      // Security
      "react/no-danger": "error",
      "react/jsx-no-target-blank": "error",
      "eslint/no-eval": "error",
      "eslint/no-new-func": "error",
      "eslint/no-script-url": "error",
      "eslint/no-restricted-globals": [
        "error",
        { name: "fetch", message: "Go through ~/api." },
        { name: "EventSource", message: "Go through ~/api." },
      ],

      // React Compiler
      "react/exhaustive-effect-dependencies": "error",
      "react/globals": "error",
      "react/hooks": "error",
      "react/immutability": "error",
      "react/purity": "error",
      "react/refs": "error",
      "react/set-state-in-effect": "error",
      "react/set-state-in-render": "error",
      "react/static-components": "error",
      "react/unsupported-syntax": "error",
      "react/no-array-index-key": "error",
      "react/no-unstable-nested-components": "error",
      // The compiler memoizes context values.
      "react/jsx-no-constructed-context-values": "off",
      // Automatic JSX runtime: React does not need to be in scope.
      "react/react-in-jsx-scope": "off",

      // TypeScript
      "typescript/no-explicit-any": "error",
      "eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],

      // Our rules
      "brume/no-manual-memo": "error",
      "brume/no-unreasoned-effect": "error",
      "brume/require-suppression-reason": "error",

      // Components
      "shadcn/no-raw-colors": "error",
      "shadcn/no-unknown-classes": "error",
    },
    overrides: [
      {
        files: ["src/**"],
        excludeFiles: ["src/components/ui/**"],
        rules: {
          "shadcn/no-restyle": ["error", { allow: ["layout"] }],
          "shadcn/no-arbitrary-values": "error",
          "shadcn/require-static-classes": "error",
          "eslint/no-restricted-imports": layerRules(),
        },
      },
      {
        files: ["src/routes/**"],
        rules: {
          "eslint/no-restricted-imports": layerRules({
            group: ["react"],
            importNames: ["useState", "useReducer", "useEffect", "useLayoutEffect", "useRef"],
            message: "A route assembles the page: state belongs in a component.",
          }),
        },
      },
      {
        files: ["src/components/**", "src/hooks/**", "src/api/**"],
        excludeFiles: ["src/components/ui/**"],
        rules: {
          "eslint/no-restricted-imports": layerRules({
            group: ["~/routes/*"],
            message: "Routes import components, not the other way around.",
          }),
        },
      },
      {
        files: ["src/helpers/**"],
        rules: {
          "eslint/no-restricted-imports": layerRules({
            group: [
              "react",
              "react-dom",
              "@tanstack/*",
              "~/components/*",
              "~/hooks/*",
              "~/routes/*",
              "~/api/*",
            ],
            allowTypeImports: true,
            message: "A helper is a pure function: no React, no data access, no components.",
          }),
        },
      },
      { files: ["src/api/**"], rules: { "eslint/no-restricted-globals": "off" } },
    ],
  },
  test: {
    retry: 0,
    allowOnly: !process.env.CI,
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          environment: "node",
          include: ["src/**/*.test.ts", "build/**/*.test.ts", "oxlint-plugin/**/*.test.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "browser",
          include: ["src/**/*.browser.test.tsx"],
          browser: {
            enabled: true,
            headless: true,
            provider: playwright(),
            instances: [{ browser: "chromium" }],
          },
        },
      },
    ],
  },
});
