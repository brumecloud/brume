"use client";

import { MotionConfig } from "motion/react";
import type { ReactNode } from "react";

/** Motion animations, like the icons', follow the visitor's reduced motion setting. */
export function MotionProvider({ children }: { readonly children: ReactNode }) {
  return <MotionConfig reducedMotion="user">{children}</MotionConfig>;
}
