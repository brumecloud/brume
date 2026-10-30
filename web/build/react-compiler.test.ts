import { globSync, readFileSync } from "node:fs";

import { transformSync } from "@babel/core";
import reactCompiler, { type LoggerEvent } from "babel-plugin-react-compiler";
import { describe, expect, it } from "vite-plus/test";

// The compiler silently skips any function it cannot prove correct, and the lint
// does not catch every case: compile every file of src/.
const SKIPPED = new Set<LoggerEvent["kind"]>(["CompileError", "CompileSkip", "PipelineError"]);

const files = globSync("src/**/*.{ts,tsx}", {
  cwd: new URL("..", import.meta.url),
  exclude: ["src/**/*.test.ts", "src/**/*.test.tsx", "src/routeTree.gen.ts"],
});

function compilerEvents(file: string): LoggerEvent[] {
  const events: LoggerEvent[] = [];
  transformSync(readFileSync(new URL(`../${file}`, import.meta.url), "utf8"), {
    filename: file,
    babelrc: false,
    configFile: false,
    parserOpts: { plugins: ["typescript", "jsx"] },
    plugins: [
      [
        reactCompiler,
        {
          panicThreshold: "none",
          logger: { logEvent: (_: unknown, event: LoggerEvent) => events.push(event) },
        },
      ],
    ],
  });
  return events;
}

describe("React Compiler", () => {
  it.each(files)("compiles every function of %s", (file) => {
    const skipped = compilerEvents(file).filter((event) => SKIPPED.has(event.kind));

    expect(skipped).toEqual([]);
  });
});
