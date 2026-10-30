import { defineRule } from "vite-plus/lint/plugins";

import { calleeName } from "./callee-name.ts";

const EFFECT_HOOKS = new Set(["useEffect", "useLayoutEffect", "useInsertionEffect"]);

export const noUnreasonedEffect = defineRule({
  meta: {
    type: "suggestion",
    messages: {
      effect:
        "{{name}}: compute during render, act in the event handler, or load with TanStack Query. " +
        "An effect that syncs with something outside React is kept with " +
        "`oxlint-disable-next-line brume/no-unreasoned-effect -- <why>`.",
    },
  },
  create(context) {
    return {
      CallExpression(node) {
        const name = calleeName(node.callee);
        if (name && EFFECT_HOOKS.has(name)) {
          context.report({ node, messageId: "effect", data: { name } });
        }
      },
    };
  },
});
