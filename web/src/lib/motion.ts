// Curves and durations from .claude/skills/improve-animations/AUDIT.md: a strong ease-out
// for what enters, under 300 ms for UI.
const EASE_OUT = [0.23, 1, 0.32, 1] as const;

export const PANEL_TRANSITION = { duration: 0.25, ease: EASE_OUT } as const;
