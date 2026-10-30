import { defineRule } from "vite-plus/lint/plugins";

const LINT_SUPPRESSION = /^(?:eslint|oxlint)-disable(?:-next-line|-line)?\b/;
const LINT_REASON = /\s--\s*\S/;
const TYPE_SUPPRESSION = /^@ts-(?:expect-error|ignore|nocheck)\b(.*)$/s;

export const requireSuppressionReason = defineRule({
  meta: {
    type: "problem",
    messages: {
      lint: "A lint suppression says why: `… -- <reason>`.",
      type: "A type suppression says why: `@ts-expect-error <reason>`.",
    },
  },
  create(context) {
    return {
      Program() {
        for (const comment of context.sourceCode.getAllComments()) {
          const text = comment.value.trim();
          if (LINT_SUPPRESSION.test(text) && !LINT_REASON.test(text)) {
            context.report({ loc: comment.loc, messageId: "lint" });
          }
          const typeSuppression = TYPE_SUPPRESSION.exec(text);
          if (typeSuppression && !typeSuppression[1]?.replace(/^[\s:-]+/, "")) {
            context.report({ loc: comment.loc, messageId: "type" });
          }
        }
      },
    };
  },
});
