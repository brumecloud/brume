import { definePlugin } from "vite-plus/lint/plugins";

import { noManualMemo } from "./no-manual-memo.ts";
import { noUnreasonedEffect } from "./no-unreasoned-effect.ts";
import { requireSuppressionReason } from "./require-suppression-reason.ts";

export default definePlugin({
  meta: { name: "brume" },
  rules: {
    "no-manual-memo": noManualMemo,
    "no-unreasoned-effect": noUnreasonedEffect,
    "require-suppression-reason": requireSuppressionReason,
  },
});
