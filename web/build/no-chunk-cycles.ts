import type { Plugin } from "vite-plus";

// A cycle between chunks can run a chunk before its dependencies and leave a
// blank page: the build fails if it finds one.
export function noChunkCycles(): Plugin {
  return {
    name: "brume:no-chunk-cycles",
    apply: "build",
    generateBundle(_options, bundle) {
      const graph = new Map<string, readonly string[]>();
      for (const output of Object.values(bundle)) {
        if (output.type === "chunk") {
          graph.set(output.fileName, output.imports);
        }
      }
      const cycle = findChunkCycle(graph);
      if (cycle) {
        this.error(`Cycle between chunks: ${cycle.join(" → ")}`);
      }
    },
  };
}

export function findChunkCycle(
  graph: ReadonlyMap<string, readonly string[]>,
): string[] | undefined {
  const done = new Set<string>();
  const path: string[] = [];

  const visit = (chunk: string): string[] | undefined => {
    const start = path.indexOf(chunk);
    if (start !== -1) {
      return [...path.slice(start), chunk];
    }
    if (done.has(chunk) || !graph.has(chunk)) {
      return undefined;
    }
    path.push(chunk);
    for (const imported of graph.get(chunk) ?? []) {
      const cycle = visit(imported);
      if (cycle) {
        return cycle;
      }
    }
    path.pop();
    done.add(chunk);
    return undefined;
  };

  for (const chunk of graph.keys()) {
    const cycle = visit(chunk);
    if (cycle) {
      return cycle;
    }
  }
  return undefined;
}
