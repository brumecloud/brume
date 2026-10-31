import { describe, it } from "vite-plus/test";
import { RuleTester } from "vite-plus/lint/plugins-dev";

import { noManualMemo } from "./no-manual-memo.ts";
import { noUnreasonedEffect } from "./no-unreasoned-effect.ts";
import { requireSuppressionReason } from "./require-suppression-reason.ts";

RuleTester.describe = describe;
RuleTester.it = it;

const tester = new RuleTester({
  languageOptions: { sourceType: "module", parserOptions: { lang: "ts" } },
});

tester.run("no-manual-memo", noManualMemo, {
  valid: ["const total = items.length;", "useState(0);", '"use strict";'],
  invalid: [
    { code: "useMemo(() => a + b, [a, b]);", errors: [{ messageId: "hook" }] },
    { code: "React.useCallback(() => {}, []);", errors: [{ messageId: "hook" }] },
    { code: 'function C() { "use memo"; }', errors: [{ messageId: "directive" }] },
    { code: 'function C() { "use no memo"; }', errors: [{ messageId: "directive" }] },
  ],
});

tester.run("no-unreasoned-effect", noUnreasonedEffect, {
  valid: ["useState(0);", "effect();"],
  invalid: [
    { code: "useEffect(() => {}, []);", errors: [{ messageId: "effect" }] },
    { code: "React.useLayoutEffect(() => {});", errors: [{ messageId: "effect" }] },
    { code: "useInsertionEffect(() => {});", errors: [{ messageId: "effect" }] },
  ],
});

tester.run("require-suppression-reason", requireSuppressionReason, {
  valid: [
    "// oxlint-disable-next-line no-eval -- demo\neval(x);",
    "/* eslint-disable no-console -- debug script */",
    "// @ts-expect-error the library declares this type wrong\nconst a: number = 'a';",
    "// @ts-expect-error: the library declares this type wrong\nconst a: number = 'a';",
    "// a regular comment",
    "// oxlint-enable no-eval",
  ],
  invalid: [
    { code: "// oxlint-disable-next-line no-eval\neval(x);", errors: [{ messageId: "lint" }] },
    { code: "/* eslint-disable */", errors: [{ messageId: "lint" }] },
    { code: "// oxlint-disable-line no-eval --\neval(x);", errors: [{ messageId: "lint" }] },
    { code: "// @ts-expect-error\nconst a: number = 'a';", errors: [{ messageId: "type" }] },
    { code: "// @ts-ignore :\nconst a: number = 'a';", errors: [{ messageId: "type" }] },
  ],
});
