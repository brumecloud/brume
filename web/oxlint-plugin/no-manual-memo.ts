import { defineRule } from "vite-plus/lint/plugins";

import { calleeName } from "./callee-name.ts";

const MEMO_HOOKS = new Set(["useMemo", "useCallback"]);
const MEMO_DIRECTIVES = new Set(["use memo", "use no memo"]);

export const noManualMemo = defineRule({
  meta: {
    type: "problem",
    messages: {
      hook: "No {{name}}: the React Compiler memoizes for us.",
      directive: 'No "{{directive}}" directive: the React Compiler runs on all of src/.',
    },
  },
  create(context) {
    return {
      CallExpression(node) {
        const name = calleeName(node.callee);
        if (name && MEMO_HOOKS.has(name)) {
          context.report({ node, messageId: "hook", data: { name } });
        }
      },
      ExpressionStatement(node) {
        if (node.directive && MEMO_DIRECTIVES.has(node.directive)) {
          context.report({ node, messageId: "directive", data: { directive: node.directive } });
        }
      },
    };
  },
});
