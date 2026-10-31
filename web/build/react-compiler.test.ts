import { globSync, readFileSync } from "node:fs";

import { transformSync } from "@babel/core";
import reactCompiler, { type LoggerEvent } from "babel-plugin-react-compiler";
import { describe, expect, it } from "vite-plus/test";

// The compiler silently skips any function it cannot prove correct, and the lint
// does not catch every case: compile every file of src/.
const SKIPPED = new Set<LoggerEvent["kind"]>(["CompileError", "CompileSkip", "PipelineError"]);

// The one deliberate exception: a component that calls a library the compiler cannot memoize
// without stale UI. The compiler says so and leaves it uncompiled, which is correct. Each such
// component is listed here with its library, and keeps nothing else than the library glue.
const INCOMPATIBLE_LIBRARY_USERS = new Map([
  ["src/components/logs/log-list.tsx", "@tanstack/react-virtual"],
]);

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

function isDeliberate(file: string, event: LoggerEvent): boolean {
  return (
    event.kind === "CompileError" &&
    String(event.detail.category) === "IncompatibleLibrary" &&
    INCOMPATIBLE_LIBRARY_USERS.has(file)
  );
}

describe("React Compiler", () => {
  it.each(files)("compiles every function of %s", (file) => {
    const skipped = compilerEvents(file).filter(
      (event) => SKIPPED.has(event.kind) && !isDeliberate(file, event),
    );

    expect(skipped).toEqual([]);
  });

  it.each([...INCOMPATIBLE_LIBRARY_USERS])("still needs the exception for %s (%s)", (file) => {
    expect(compilerEvents(file).some((event) => isDeliberate(file, event))).toBe(true);
  });
});
