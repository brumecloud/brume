import type { ESTree } from "vite-plus/lint/plugins";

// `useMemo(…)` as well as `React.useMemo(…)`.
export function calleeName(callee: ESTree.CallExpression["callee"]): string | undefined {
  if (callee.type === "Identifier") {
    return callee.name;
  }
  if (callee.type === "MemberExpression" && callee.property.type === "Identifier") {
    return callee.property.name;
  }
  return undefined;
}
